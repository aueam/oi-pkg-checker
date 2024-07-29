use crate::{
    cli::{Args, Commands},
    logger::Logger,
};
use clap::Parser;
use colored::Colorize;
use fmri::FMRI;
use log::{debug, error, info, warn, LevelFilter};
use oi_pkg_checker_core::{
    assets::{catalogs_c::load_catalog_c, open_indiana_oi_userland_git::load_git},
    packages::{
        components::Components,
        dependency_type::DependencyTypes,
        dependency_type::DependencyTypes::{Build, SystemBuild, SystemTest, Test},
        rev_depend_type::RevDependType::*,
    },
    report,
};
use std::{
    path::{Path, PathBuf},
    process::exit,
};

mod cli;
mod logger;

static LOGGER: Logger = Logger;

fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let data_path = "data.bin";
    let components_path = PathBuf::from("assets/oi-userland/components");

    if let Some(subcommand) = Args::parse().command {
        match subcommand {
            Commands::PrintProblems { debug } => {
                debug_on(debug);
                let mut a = Components::deserialize(data_path).unwrap();

                report(&mut a.problems);
            }
            Commands::CheckFMRI {
                fmri,
                debug,
                hide_renamed,
            } => {
                debug_on(debug);

                let fmri = &FMRI::parse_raw(&fmri).unwrap();

                info!("fmri: {}", fmri);

                let components = match Path::new(data_path).exists() {
                    false => {
                        error!("{} doesn't exist", data_path);
                        exit(1);
                    }
                    _ => Components::deserialize(data_path).unwrap(),
                };

                let package = match components.get_package_by_fmri(fmri) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("{}", e);
                        exit(1);
                    }
                }
                .borrow();

                if package.is_obsolete() {
                    info!("package is obsolete");
                }

                if package.is_renamed() {
                    info!("package is renamed");
                }

                let runtime_deps = package.get_runtime_dependents();
                if !runtime_deps.is_empty() {
                    let mut require: Vec<String> = Vec::new();
                    let mut optional: Vec<String> = Vec::new();
                    let mut incorporate: Vec<String> = Vec::new();
                    let mut require_any: Vec<String> = Vec::new();
                    let mut conditional_fmri: Vec<String> = Vec::new();
                    let mut conditional_predicate: Vec<String> = Vec::new();
                    let mut group: Vec<String> = Vec::new();

                    let fr = |f: &FMRI| -> bool {
                        components
                            .get_package_by_fmri(f)
                            .unwrap()
                            .borrow()
                            .is_renamed()
                    };

                    for dep_b in runtime_deps {
                        let (a, f, r) = match dep_b {
                            Require(f) => (&mut require, f, fr(f)),
                            Optional(f) => (&mut optional, f, fr(f)),
                            Incorporate(f) => (&mut incorporate, f, fr(f)),
                            RequireAny(f) => (&mut require_any, f, fr(f)),
                            ConditionalFmri(f) => (&mut conditional_fmri, f, fr(f)),
                            ConditionalPredicate(f) => (&mut conditional_predicate, f, fr(f)),
                            Group(f) => (&mut group, f, fr(f)),
                        };

                        if hide_renamed && r {
                            continue;
                        }

                        a.push(
                            format!("{}{}", f, if r { " (renamed)" } else { "" })
                                .trim_start_matches("pkg://openindiana.org/")
                                .to_owned(),
                        )
                    }

                    let process = |mut ds: Vec<String>, label: &str| {
                        if !ds.is_empty() {
                            ds.sort();
                            ds.dedup();
                            info!("  {}", label);
                            for d in ds {
                                info!("    {}", d);
                            }
                        }
                    };

                    info!("{}", "RUNTIME dependents:".bold());
                    process(require, "Require");
                    process(optional, "Optional");
                    process(incorporate, "Incorporate");
                    process(require_any, "RequireAny");
                    process(conditional_fmri, "Conditional (FMRI)");
                    process(conditional_predicate, "Conditional (Predicate)");
                    process(group, "Group");
                }

                let check_deps = |dt: DependencyTypes, label: &str| {
                    let build = package.get_git_dependents(dt).unwrap();
                    if !build.is_empty() {
                        info!("{}", format!("{} (component/s) dependents:", label).bold());

                        let mut deps = build
                            .iter()
                            .map(|a| a.borrow().get_name().clone())
                            .collect::<Vec<String>>();
                        deps.sort();
                        deps.dedup();

                        for d in deps {
                            info!("    {}", d)
                        }
                    }
                };

                check_deps(Build, "BUILD");
                check_deps(SystemBuild, "SYSTEMBUILD");
                check_deps(Test, "TEST");
                check_deps(SystemTest, "SYSTEMTEST");

                if let Some(c) = package.is_in_component() {
                    let component = c.borrow();
                    info!("component name: {}", component.get_name());
                } else {
                    warn!("missing component for package");
                }
            }
            Commands::Run { catalog, debug } => {
                debug_on(debug);

                let mut components = Components::default();

                if catalog.is_empty() {
                    warn!("no catalog found")
                }

                for path in catalog {
                    load_catalog_c(&mut components, &path).unwrap();
                }

                match load_git(&mut components, &components_path) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("failed to load git: {}", e);
                        exit(0);
                    }
                };

                components.check_problems().unwrap();

                report(&mut components.problems);

                components.serialize(data_path).unwrap();
            }
        }
    }
}

fn debug_on(debug: bool) {
    if debug {
        log::set_max_level(LevelFilter::Debug);
        debug!("debug is on");
    }
}
