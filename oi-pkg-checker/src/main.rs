mod logger;
mod cli;

use std::ffi::OsStr;
use std::fs::{File, remove_file};
use std::io::{Write};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use clap::Parser;
use log::{LevelFilter, debug, error, info};
use reqwest::blocking::get;
use reqwest::IntoUrl;
use fmri::FMRI;
use crate::cli::{Args, Commands, DataCommands};
use crate::logger::Logger;
use oi_pkg_checker_core::{
    MissingComponentForPackageList, NonExistingRequiredPackageList, ObsoletedRequiredPackageList,
    PartlyObsoletedRequiredPackageList, Problems, RenamedPackageInComponentList, report,
    UnRunnableMakeCommandList, UselessComponentsList, PackageVersions, DependTypes, Components,
    Assets, ComponentPackagesList,
};


static LOGGER: Logger = Logger;

fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let data_path = "data.bin";
    let problems_path = "problems.bin";

    let args = Args::parse();
    match &args.command {
        Some(subcommand) => {
            match subcommand {
                Commands::PrintProblems { debug } => {
                    debug_on(debug);

                    let (
                        a,
                        b,
                        c,
                        d,
                        e,
                        f,
                        g,
                        h
                    ) = Problems::deserialize(problems_path).get();

                    let components = &Components::deserialize("data.bin");

                    report(
                        a,
                        b,
                        c,
                        d,
                        e,
                        f,
                        g,
                        h,
                        components,
                    );
                    exit(0);
                }
                Commands::CheckFMRI {
                    repo_path
                    , fmri
                    , debug
                } => {
                    debug_on(debug);

                    let fmri = &FMRI::parse_raw(fmri);

                    match Path::new(data_path).exists() {
                        false => {
                            error!("data doesn't exist");
                            exit(1);
                        }
                        _ => {}
                    }

                    let components = Components::deserialize(data_path);

                    if !components.check_if_fmri_exists_as_package(fmri) {
                        error!("package with name '{}' doesn't exist or is obsoleted", fmri.get_package_name_as_ref_string());
                    }

                    match components.get_dependencies_with_fmri(fmri) {
                        Some(dependencies) => {
                            info!("fmri {} is required by package/s:", fmri);
                            for (fmri, dependency_type, dependency) in dependencies {
                                let d_type = match dependency.get_ref() {
                                    DependTypes::Require(_) => "require",
                                    DependTypes::Optional(_) => "optional",
                                    DependTypes::Incorporate(_) => "incorporate",
                                    DependTypes::RequireAny(_) => "require-any",
                                    DependTypes::Conditional(_, _) => "conditional",
                                    DependTypes::Group(_) => "group",
                                    _ => unimplemented!()
                                };

                                info!("\ttype: {}, dependency: {}, package: {}", d_type, dependency_type, fmri);
                            }
                        }
                        None => {
                            info!("fmri {} isn't require by any package", fmri);
                        }
                    }

                    let components_path = PathBuf::from(format!("{}/components", repo_path.to_string_lossy()));

                    let component_packages_list = ComponentPackagesList::new(
                        &components_path.clone(),
                        &PathBuf::from((components_path.to_string_lossy() + "/encumbered").to_string()),
                    );

                    match component_packages_list.get_component_name_of_package_versions(
                        &PackageVersions::new(fmri.clone())
                    ) {
                        Ok(name) => {
                            info!("component name: {}", name)
                        }
                        Err(_) => {
                            info!("missing component for package")
                        }
                    }

                    exit(0);
                }
                Commands::Data { data_commands } => {
                    match data_commands {
                        None => {}
                        Some(command) => {
                            match command {
                                DataCommands::UpdateAssets {
                                    catalog,
                                    encumbered_catalog,
                                    repo_path
                                } => {
                                    download_file("https://pkg.openindiana.org/hipster/catalog/1/catalog.dependency.C", catalog);
                                    download_file("https://pkg.openindiana.org/hipster-encumbered/catalog/1/catalog.dependency.C", encumbered_catalog);

                                    let command = Command::new("sh")
                                        .arg("-c")
                                        .arg(format!("cd {:?} && git pull", repo_path))
                                        .output()
                                        .expect("failed to run command");

                                    let exit_code = command.status.code().unwrap();
                                    if exit_code != 0 {
                                        error!("{:?}", String::from_utf8_lossy(&command.stderr));
                                        panic!("Cannot update OpenIndiana/oi-userland repo, exit code: {}", exit_code)
                                    }
                                    exit(0);
                                }
                                DataCommands::Run {
                                    catalog
                                    , repo_path
                                    , debug
                                } => {
                                    debug_on(debug);

                                    let mut missing_component_for_package_list = MissingComponentForPackageList::new();
                                    let mut renamed_package_in_component_list = RenamedPackageInComponentList::new();
                                    let mut un_runnable_make_command_list = UnRunnableMakeCommandList::new();
                                    let mut non_existing_required_package_list = NonExistingRequiredPackageList::new();
                                    let mut obsolete_required_package_list = ObsoletedRequiredPackageList::new();
                                    let mut partly_obsolete_required_package_list = PartlyObsoletedRequiredPackageList::new();
                                    let mut useless_components_list = UselessComponentsList::new();


                                    let mut components = Components::new();

                                    let components_path = PathBuf::from(format!("{}/components", repo_path.to_string_lossy()));

                                    let catalog_path = &components_path.clone();

                                    let encumbered_catalog_path = &PathBuf::from((components_path.to_string_lossy() + "/encumbered").to_string());

                                    match components.load(Assets::Catalogs(
                                        catalog.clone()), catalog_path, encumbered_catalog_path) {
                                        Ok(_) => {}
                                        Err(problem) => match problem {
                                            Ok(problems) => renamed_package_in_component_list += problems,
                                            Err(_) => {}
                                        }
                                    }

                                    match components.load(Assets::OpenIndianaOiUserlandGit {
                                        load_component_list: true,
                                        load_build_dependencies: true,
                                        load_test_dependencies: true,
                                        load_system_build_dependencies: true,
                                        load_system_test_dependencies: true,
                                    }, catalog_path, encumbered_catalog_path) {
                                        Ok(_) => {}
                                        Err(error) => match error {
                                            Ok(_) => {}
                                            Err((missing, renamed, un_runnable)) => {
                                                missing_component_for_package_list += missing;
                                                renamed_package_in_component_list += renamed;
                                                un_runnable_make_command_list += un_runnable;
                                            }
                                        }
                                    }

                                    match components.check_dependency_validity() {
                                        Ok(_) => {}
                                        Err((non_existing, obsolete, partly_obsolete)) => {
                                            non_existing_required_package_list += non_existing;
                                            obsolete_required_package_list += obsolete;
                                            partly_obsolete_required_package_list += partly_obsolete;
                                        }
                                    }

                                    match components.get_useless_components() {
                                        Ok(_) => {}
                                        Err(useless) => {
                                            useless_components_list += useless
                                        }
                                    }

                                    let renamed_needs_renamed_list = components.check_if_renamed_needs_renamed();

                                    components.clone().serialize(data_path);

                                    report(
                                        missing_component_for_package_list.clone(),
                                        renamed_needs_renamed_list.clone(),
                                        renamed_package_in_component_list.clone(),
                                        un_runnable_make_command_list.clone(),
                                        non_existing_required_package_list.clone(),
                                        obsolete_required_package_list.clone(),
                                        partly_obsolete_required_package_list.clone(),
                                        useless_components_list.clone(),
                                        &components,
                                    );


                                    let problems = Problems::new(
                                        missing_component_for_package_list,
                                        renamed_needs_renamed_list,
                                        renamed_package_in_component_list,
                                        un_runnable_make_command_list,
                                        non_existing_required_package_list,
                                        obsolete_required_package_list,
                                        partly_obsolete_required_package_list,
                                        useless_components_list,
                                    );

                                    problems.serialize(problems_path);

                                    exit(0);
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }

    // debug_on(&true);
    // let components = Components::deserialize("data.bin");
    // println!("{}", components);

    exit(0);
}

fn debug_on(debug: &bool) {
    if *debug {
        log::set_max_level(LevelFilter::Debug);
        debug!("debug is on");
    }
}

fn download_file<T: IntoUrl, P: AsRef<Path> + AsRef<OsStr> + ?Sized>(url: T, path: &P) {
    if Path::new(&path).exists() {
        remove_file(&path).expect("Failed to remove existing file");
    }

    File::create(&path)
        .expect("Failed to create file")
        .write_all(
            get(url)
                .expect("Failed to download file")
                .text()
                .unwrap()
                .as_bytes()
        )
        .expect("TODO: panic message");
}