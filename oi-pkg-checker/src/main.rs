use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use fmri::FMRI;
use log::{debug, error, info, LevelFilter};

use oi_pkg_checker_core::{
    report, AssetTypes, ComponentPackagesList, Components, DependTypes, PackageVersions, Problems,
};

use crate::{
    cli::{Args, Commands},
    logger::Logger,
};

mod cli;
mod logger;

static LOGGER: Logger = Logger;

fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let data_path = "data.bin";
    let problems_path = "problems.bin";
    let components_path = &PathBuf::from("assets/oi-userland/components");

    match &Args::parse().command {
        Some(subcommand) => match subcommand {
            Commands::PrintProblems { debug } => {
                debug_on(debug);
                report(&mut Problems::deserialize(problems_path).unwrap());
                exit(0);
            }
            Commands::CheckFMRI { fmri, debug } => {
                debug_on(debug);

                let fmri = &FMRI::parse_raw(fmri).unwrap();

                let components = match Path::new(data_path).exists() {
                    false => {
                        error!("{} doesn't exist", data_path);
                        exit(1);
                    }
                    _ => Components::deserialize(data_path),
                };

                if !components.check_if_fmri_exists_as_package(fmri) {
                    error!(
                        "package with name '{}' doesn't exist or is obsoleted",
                        fmri.get_package_name_as_ref_string()
                    );
                }

                if let Some(dependencies) = components.get_dependencies_with_fmri(fmri) {
                    info!("fmri {} is required by:", fmri);
                    for (fmri, dependency_type, dependency) in dependencies {
                        let d_type = match dependency.get_ref() {
                            DependTypes::Require(_) => "require",
                            DependTypes::Optional(_) => "optional",
                            DependTypes::Incorporate(_) => "incorporate",
                            DependTypes::RequireAny(_) => "require-any",
                            DependTypes::Conditional(_, _) => "conditional",
                            DependTypes::Group(_) => "group",
                            _ => unimplemented!(),
                        };

                        info!(
                            "\ttype: {}, dependency: {}, package: {}",
                            d_type, dependency_type, fmri
                        );
                    }
                } else {
                    info!("fmri {} is not required by any package", fmri);
                }

                if let Some(name) = ComponentPackagesList::new(components_path)
                    .get_component_packages_of_package_versions(
                        &mut Problems::new(),
                        &PackageVersions::new(fmri.clone()),
                    )
                    .map(|a| a.component_name)
                {
                    info!("component name: {}", name)
                } else {
                    info!("missing component for package")
                }

                exit(0);
            }
            Commands::Run { catalog, debug } => {
                debug_on(debug);

                let mut problems = Problems::new();
                let mut components = Components::new();

                components.load(
                    &mut problems,
                    AssetTypes::Catalogs(catalog.clone()),
                    components_path,
                );
                components.load(
                    &mut problems,
                    AssetTypes::OpenIndianaOiUserlandGit,
                    components_path,
                );

                components.check_dependency_validity(&mut problems);
                components.get_useless_components(&mut problems);
                components.check_if_renamed_needs_renamed(&mut problems);

                report(&mut problems);

                components.serialize(data_path);

                problems
                    .serialize(problems_path)
                    .expect("TODO: panic message");

                exit(0);
            }
        },
        None => {}
    }

    exit(0);
}

fn debug_on(debug: &bool) {
    if *debug {
        log::set_max_level(LevelFilter::Debug);
        debug!("debug is on");
    }
}
