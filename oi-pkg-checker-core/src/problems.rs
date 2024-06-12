use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use bincode::{deserialize, serialize};
use fmri::FMRI;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::packages::{
    components::Components, depend_types::DependTypes, dependency_type::DependencyTypes,
};
use crate::problems::Problem::{
    MissingComponentForPackage, NonExistingRequiredPackage, ObsoletedPackageInComponent,
    ObsoletedRequiredPackage, PartlyObsoletedRequiredPackage, RenamedNeedsRenamed,
    RenamedPackageInComponent, UnRunnableMakeCommand, UselessComponent,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Problem {
    MissingComponentForPackage(FMRI),
    RenamedNeedsRenamed(FMRI, FMRI),
    RenamedPackageInComponent(FMRI, String),
    ObsoletedPackageInComponent(FMRI, String),
    UnRunnableMakeCommand(String, PathBuf),
    // TODO: divide into two (renamed and not renamed)
    NonExistingRequiredPackage(DependTypes, DependencyTypes, FMRI, bool),
    // TODO: divide into two (renamed and not renamed)
    ObsoletedRequiredPackage(DependTypes, DependencyTypes, FMRI, bool),
    // TODO: divide into two (renamed and not renamed)
    PartlyObsoletedRequiredPackage(DependTypes, DependencyTypes, FMRI, bool),
    UselessComponent(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Problems(Vec<Problem>);

impl Default for Problems {
    fn default() -> Self {
        Self::new()
    }
}

impl Problems {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn get_ref(&self) -> &Vec<Problem> {
        &self.0
    }

    pub fn add_problem(&mut self, mut problem: Problem) {
        match &mut problem {
            MissingComponentForPackage(fmri) => {
                fmri.remove_version();
            }
            RenamedNeedsRenamed(fmri_a, fmri_b) => {
                fmri_a.remove_version();
                fmri_b.remove_version();
            }
            RenamedPackageInComponent(fmri, _component_name) => {
                fmri.remove_version();
            }
            ObsoletedPackageInComponent(fmri, _component_name) => {
                fmri.remove_version();
            }
            UnRunnableMakeCommand(_command, _component_path) => {}
            NonExistingRequiredPackage(_depend_type, _dependency_type, required_by, _renamed) => {
                required_by.remove_version();
            }
            ObsoletedRequiredPackage(_depend_type, _dependency_type, required_by, _renamed) => {
                required_by.remove_version();
            }
            PartlyObsoletedRequiredPackage(
                _depend_type,
                _dependency_type,
                required_by,
                _renamed,
            ) => {
                required_by.remove_version();
            }
            UselessComponent(_component_name) => {}
        }

        if !self.0.contains(&problem) {
            self.0.push(problem)
        }
    }

    pub fn serialize<P: AsRef<Path> + ?Sized + std::fmt::Display>(
        &self,
        path: &P,
    ) -> Result<(), String> {
        File::create(path)
            .unwrap()
            .write_all(&serialize(self).map_err(|e| {
                format!("failed to serialize file with problems into binary: {}", e)
            })?)
            .map_err(|e| format!("failed to write problems to {}: {}", path, e))?;
        Ok(())
    }

    pub fn deserialize<P: AsRef<Path> + ?Sized + std::fmt::Display>(
        path: &P,
    ) -> Result<Self, String> {
        let data = &mut Vec::new();
        File::open(path)
            .map_err(|e| format!("failed to open file with problems: {}", e))?
            .read_to_end(data)
            .map_err(|e| format!("failed to read problems: {}", e))?;
        deserialize(data).map_err(|e| format!("failed to deserialize data from {}: {}", path, e))
    }

    fn sort(&mut self) {
        let priority = |item: &Problem| -> usize {
            match item {
                UselessComponent(_) => 0,
                PartlyObsoletedRequiredPackage(_, _, _, _) => 1,
                MissingComponentForPackage(_) => 2,
                NonExistingRequiredPackage(_, _, _, _) => 3,
                RenamedNeedsRenamed(_, _) => 4,
                RenamedPackageInComponent(_, _) => 5,
                ObsoletedPackageInComponent(_, _) => 6,
                ObsoletedRequiredPackage(_, _, _, _) => 7,
                UnRunnableMakeCommand(_, _) => 8,
            }
        };

        self.0.sort_by_key(priority)
    }

    fn count(&self) {
        let mut counter: [i16; 9] = [0; 9];
        for problem in self.get_ref() {
            match problem {
                UselessComponent(_) => counter[0] += 1,
                PartlyObsoletedRequiredPackage(_, _, _, _) => counter[1] += 1,
                MissingComponentForPackage(_) => counter[2] += 1,
                NonExistingRequiredPackage(_, _, _, _) => counter[3] += 1,
                RenamedNeedsRenamed(_, _) => counter[4] += 1,
                RenamedPackageInComponent(_, _) => counter[5] += 1,
                ObsoletedPackageInComponent(_, _) => counter[6] += 1,
                ObsoletedRequiredPackage(_, _, _, _) => counter[7] += 1,
                UnRunnableMakeCommand(_, _) => counter[8] += 1,
            }
        }

        for (problem_type, count) in counter.iter().enumerate() {
            match problem_type {
                0 => info!("Number of components that are not needed by any package: {}", count),
                1 => warn!("Number of obsoleted packages with older normal version which are needed as dependency: {}", count),
                2 => warn!("Number of packages that do not belong to a component: {}", count),
                3 => warn!("Number of non existing packages which are needed as dependency: {}", count),
                4 => error!("Number of renamed packages that need renamed packages: {}", count),
                5 => error!("Number of renamed packages which are in component: {}", count),
                6 => error!("Number of obsoleted packages which are in component: {}", count),
                7 => error!("Number of obsoleted packages which are needed as dependency: {}", count),
                8 => error!("Number of un-runnable make commands: {}", count),
                _ => panic!("invalid problem type"),
            }
        }
    }
}

pub fn report(problems: &mut Problems, components: &Components) {
    problems.sort();

    for problem in problems.get_ref() {
        match problem {
            UselessComponent(name) => info!("component {} is not needed by any package", name),
            MissingComponentForPackage(fmri) => warn!("missing component for {}", fmri),
            RenamedNeedsRenamed(fmri_a, fmri_b) => error!(
                "renamed package {} needs renamed package {}",
                fmri_a.get_package_name_as_ref_string(),
                fmri_b.get_package_name_as_ref_string()
            ),
            RenamedPackageInComponent(package, component) => error!(
                "package {} is renamed and is in component {}",
                package.get_package_name_as_ref_string(),
                component
            ),
            ObsoletedPackageInComponent(package, component) => error!(
                "package {} is obsolete and is in component {}",
                package.get_package_name_as_ref_string(),
                component
            ),
            UnRunnableMakeCommand(command, path) => error!("can't run {} in {:?}", command, path),
            NonExistingRequiredPackage(depend_type, dependency_type, required_by, renamed) => {
                let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

                // TODO: move get_component_name_by_package()
                let mut package_or_component_name = if let Some(component_name) =
                    components.get_component_name_by_package(required_by)
                {
                    "component ".to_owned() + component_name
                } else {
                    panic!("component does not exist")
                };

                let mut by_and_types = match renamed {
                    true => {
                        package_or_component_name =
                            required_by.get_package_name_as_ref_string().clone();
                        "by renamed package ".to_owned()
                    }
                    false => {
                        if dependency_type == &DependencyTypes::Runtime {
                            package_or_component_name =
                                required_by.get_package_name_as_ref_string().clone();
                        }
                        "by ".to_owned()
                    }
                };

                by_and_types.push_str(&match renamed {
                    true => match dependency_type {
                        DependencyTypes::Runtime => {
                            format!("{} (runtime, {})", package_or_component_name, name)
                        }
                        DependencyTypes::Build => {
                            format!("{} (build, {})", package_or_component_name, name)
                        }
                        DependencyTypes::Test => {
                            format!("{} (test, {})", package_or_component_name, name)
                        }
                        DependencyTypes::SystemBuild => {
                            format!("{} (system-build)", package_or_component_name)
                        }
                        DependencyTypes::SystemTest => {
                            format!("{} (system-test)", package_or_component_name)
                        }
                        DependencyTypes::None => panic!("DependencyTypes can't be None"),
                    },
                    false => match dependency_type {
                        DependencyTypes::Runtime => {
                            format!("package {} (runtime, {})", package_or_component_name, name)
                        }
                        DependencyTypes::Build => {
                            format!("{} (build, component)", package_or_component_name)
                        }
                        DependencyTypes::Test => {
                            format!("{} (test, component)", package_or_component_name)
                        }
                        DependencyTypes::SystemBuild => {
                            format!("{} (build, system)", package_or_component_name)
                        }
                        DependencyTypes::SystemTest => {
                            format!("{} (test, system)", package_or_component_name)
                        }
                        DependencyTypes::None => panic!("DependencyTypes can't be None"),
                    },
                });

                warn!(
                    "package {} doesn't exist, but is required {}",
                    fmri, by_and_types
                )
            }
            ObsoletedRequiredPackage(depend_type, dependency_type, required_by, renamed) => {
                let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

                let mut package_or_component_name = if let Some(component_name) =
                    components.get_component_name_by_package(required_by)
                {
                    "component ".to_owned() + component_name
                } else {
                    panic!("component does not exist")
                };

                let mut by_and_types = match renamed {
                    true => {
                        package_or_component_name =
                            required_by.get_package_name_as_ref_string().clone();
                        "by renamed package ".to_owned()
                    }
                    false => {
                        if dependency_type == &DependencyTypes::Runtime {
                            package_or_component_name =
                                required_by.get_package_name_as_ref_string().clone();
                        }
                        "by ".to_owned()
                    }
                };

                by_and_types.push_str(&match renamed {
                    true => match dependency_type {
                        DependencyTypes::Runtime => {
                            format!("{} (runtime, {})", package_or_component_name, name)
                        }
                        DependencyTypes::Build => {
                            format!("{} (build, component)", package_or_component_name)
                        }
                        DependencyTypes::Test => {
                            format!("{} (test, component)", package_or_component_name)
                        }
                        DependencyTypes::SystemBuild => {
                            format!("{} (system-build, system)", package_or_component_name)
                        }
                        DependencyTypes::SystemTest => {
                            format!("{} (system-test, system)", package_or_component_name)
                        }
                        DependencyTypes::None => panic!("DependencyTypes can't be None"),
                    },
                    false => match dependency_type {
                        DependencyTypes::Runtime => {
                            format!("package {} (runtime, {})", package_or_component_name, name)
                        }
                        DependencyTypes::Build => {
                            format!("{} (build, component)", package_or_component_name)
                        }
                        DependencyTypes::Test => {
                            format!("{} (test, component)", package_or_component_name)
                        }
                        DependencyTypes::SystemBuild => {
                            format!("{} (build, system)", package_or_component_name)
                        }
                        DependencyTypes::SystemTest => {
                            format!("{} (test, system)", package_or_component_name)
                        }
                        DependencyTypes::None => panic!("DependencyTypes can't be None"),
                    },
                });

                error!("obsoleted package {} is required {}", fmri, by_and_types);
            }
            PartlyObsoletedRequiredPackage(depend_type, dependency_type, required_by, renamed) => {
                let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

                let mut package_or_component_name = if let Some(component_name) =
                    components.get_component_name_by_package(required_by)
                {
                    "component ".to_owned() + component_name
                } else {
                    panic!("component does not exist")
                };

                let mut by_and_types = match renamed {
                    true => {
                        package_or_component_name =
                            required_by.get_package_name_as_ref_string().clone();
                        "by renamed package ".to_owned()
                    }
                    false => {
                        if dependency_type == &DependencyTypes::Runtime {
                            package_or_component_name =
                                required_by.get_package_name_as_ref_string().clone();
                        }
                        "by ".to_owned()
                    }
                };

                by_and_types.push_str(&match renamed {
                    true => match dependency_type {
                        DependencyTypes::Runtime => {
                            format!("{} (runtime, {})", package_or_component_name, name)
                        }
                        DependencyTypes::Build => {
                            format!("{} (build, component)", package_or_component_name)
                        }
                        DependencyTypes::Test => {
                            format!("{} (test, component)", package_or_component_name)
                        }
                        DependencyTypes::SystemBuild => {
                            format!("{} (system-build, system)", package_or_component_name)
                        }
                        DependencyTypes::SystemTest => {
                            format!("{} (system-test, system)", package_or_component_name)
                        }
                        DependencyTypes::None => panic!("DependencyTypes can't be None"),
                    },
                    false => match dependency_type {
                        DependencyTypes::Runtime => {
                            format!("package {} (runtime, {})", package_or_component_name, name)
                        }
                        DependencyTypes::Build => {
                            format!("{} (build, component)", package_or_component_name)
                        }
                        DependencyTypes::Test => {
                            format!("{} (test, component)", package_or_component_name)
                        }
                        DependencyTypes::SystemBuild => {
                            format!("{} (build, system)", package_or_component_name)
                        }
                        DependencyTypes::SystemTest => {
                            format!("{} (test, system)", package_or_component_name)
                        }
                        DependencyTypes::None => panic!("DependencyTypes can't be None"),
                    },
                });

                warn!("obsoleted package {} is required {}", fmri, by_and_types);
            }
        }
    }

    problems.count()
}
