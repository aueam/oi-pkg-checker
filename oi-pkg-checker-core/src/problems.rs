use std::path::PathBuf;

use fmri::{FMRI, Publisher};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    packages::{depend_types::DependTypes, dependency_type::DependencyTypes},
    problems::Problem::{
        MissingComponentForPackage, NonExistingPackageInPkg5, NonExistingRequired,
        NonExistingRequiredByRenamed, ObsoletedPackageInComponent, ObsoletedRequired,
        ObsoletedRequiredByRenamed, PackageInMultipleComponents, PartlyObsoletedRequired,
        PartlyObsoletedRequiredByRenamed, RenamedNeedsRenamed, RenamedPackageInComponent,
        UnRunnableMakeCommand, UselessComponent,
    },
};
use crate::problems::Problem::SamePackageHasTwoPublishers;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Problem {
    MissingComponentForPackage(FMRI),
    RenamedNeedsRenamed(FMRI, FMRI),
    RenamedPackageInComponent(FMRI, String),
    ObsoletedPackageInComponent(FMRI, String),
    UnRunnableMakeCommand(String, PathBuf),
    NonExistingRequired(DependTypes, DependencyTypes, FMRI, String),
    NonExistingRequiredByRenamed(DependTypes, DependencyTypes, FMRI),
    ObsoletedRequired(DependTypes, DependencyTypes, FMRI, String),
    ObsoletedRequiredByRenamed(DependTypes, DependencyTypes, FMRI),
    PartlyObsoletedRequired(DependTypes, DependencyTypes, FMRI, String),
    PartlyObsoletedRequiredByRenamed(DependTypes, DependencyTypes, FMRI),
    UselessComponent(String),
    PackageInMultipleComponents(FMRI, Vec<String>),
    NonExistingPackageInPkg5(FMRI, String),
    SamePackageHasTwoPublishers(FMRI, Publisher, Publisher, Option<Publisher>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Problems(Vec<Problem>);

impl Problems {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn get_ref(&self) -> &Vec<Problem> {
        &self.0
    }

    pub fn add_problem(&mut self, mut problem: Problem) {
        match &mut problem {
            MissingComponentForPackage(f)
            | NonExistingRequired(_, _, f, _)
            | NonExistingRequiredByRenamed(_, _, f)
            | ObsoletedRequired(_, _, f, _)
            | ObsoletedRequiredByRenamed(_, _, f)
            | PartlyObsoletedRequired(_, _, f, _)
            | PartlyObsoletedRequiredByRenamed(_, _, f)
            | RenamedPackageInComponent(f, _)
            | ObsoletedPackageInComponent(f, _)
            | PackageInMultipleComponents(f, _)
            | NonExistingPackageInPkg5(f, _)
            | SamePackageHasTwoPublishers(f, _, _, _) => {
                f.remove_version();
                f.remove_publisher();
            }
            RenamedNeedsRenamed(fmri_a, fmri_b) => {
                fmri_a.remove_version();
                fmri_b.remove_version();
                fmri_a.remove_publisher();
                fmri_b.remove_publisher();
            }
            UnRunnableMakeCommand(_, _) => {}
            UselessComponent(_) => {}
        }

        if !self.contains(&problem) {
            self.0.push(problem)
        }
    }

    fn contains(&self, problem: &Problem) -> bool {
        let contains_component = |depend_type: &DependTypes,
                                  dependency_type: &DependencyTypes,
                                  component_name: &String|
         -> bool {
            if dependency_type != &DependencyTypes::Runtime {
                return self.0.iter().any(|p| {
                    if let NonExistingRequired(a, b, _, c) = p {
                        if a == depend_type && b == dependency_type && c == component_name {
                            return true;
                        }
                    }

                    if let ObsoletedRequired(a, b, _, c) = p {
                        if a == depend_type && b == dependency_type && c == component_name {
                            return true;
                        }
                    }

                    if let PartlyObsoletedRequired(a, b, _, c) = p {
                        if a == depend_type && b == dependency_type && c == component_name {
                            return true;
                        }
                    }

                    false
                });
            }

            false
        };

        match problem {
            NonExistingRequired(depend_type, dependency_type, _, component_name) => {
                if contains_component(depend_type, dependency_type, component_name) {
                    return true;
                }
            }
            ObsoletedRequired(depend_type, dependency_type, _, component_name) => {
                if contains_component(depend_type, dependency_type, component_name) {
                    return true;
                }
            }
            PartlyObsoletedRequired(depend_type, dependency_type, _, component_name) => {
                if contains_component(depend_type, dependency_type, component_name) {
                    return true;
                }
            }
            _ => {}
        };

        self.0.contains(problem)
    }

    pub fn sort(&mut self) {
        let priority = |item: &Problem| -> usize {
            match item {
                UselessComponent(_) => 0,
                PartlyObsoletedRequired(_, _, _, _) => 1,
                PartlyObsoletedRequiredByRenamed(_, _, _) => 2,
                MissingComponentForPackage(_) => 3,
                NonExistingRequired(_, _, _, _) => 4,
                NonExistingRequiredByRenamed(_, _, _) => 5,
                RenamedNeedsRenamed(_, _) => 6,
                RenamedPackageInComponent(_, _) => 7,
                ObsoletedPackageInComponent(_, _) => 8,
                ObsoletedRequired(_, _, _, _) => 9,
                ObsoletedRequiredByRenamed(_, _, _) => 10,
                UnRunnableMakeCommand(_, _) => 11,
                PackageInMultipleComponents(_, _) => 12,
                NonExistingPackageInPkg5(_, _) => 13,
                SamePackageHasTwoPublishers(_, _, _, _) => 14,
            }
        };

        self.0.sort_by_key(priority)
    }

    fn count(&self) {
        let mut counter: [i16; 15] = [0; 15];
        for problem in self.get_ref() {
            match problem {
                UselessComponent(_) => counter[0] += 1,
                PartlyObsoletedRequired(_, _, _, _) => counter[1] += 1,
                PartlyObsoletedRequiredByRenamed(_, _, _) => counter[2] += 1,
                MissingComponentForPackage(_) => counter[3] += 1,
                NonExistingRequired(_, _, _, _) => counter[4] += 1,
                NonExistingRequiredByRenamed(_, _, _) => counter[5] += 1,
                RenamedNeedsRenamed(_, _) => counter[6] += 1,
                RenamedPackageInComponent(_, _) => counter[7] += 1,
                ObsoletedPackageInComponent(_, _) => counter[8] += 1,
                ObsoletedRequired(_, _, _, _) => counter[9] += 1,
                ObsoletedRequiredByRenamed(_, _, _) => counter[10] += 1,
                UnRunnableMakeCommand(_, _) => counter[11] += 1,
                PackageInMultipleComponents(_, _) => counter[12] += 1,
                NonExistingPackageInPkg5(_, _) => counter[13] += 1,
                SamePackageHasTwoPublishers(_, _, _, _) => counter[14] += 1,
            }
        }

        for (problem_type, count) in counter.iter().enumerate() {
            match problem_type {
                0 => info!("Number of components that are not needed by any package: {}", count),
                1 => warn!("Number of obsoleted packages with older normal version which are needed as dependency: {}", count),
                2 => warn!("Number of obsoleted packages with older normal version which are needed as dependency in renamed package: {}", count),
                3 => warn!("Number of packages that do not belong to a component: {}", count),
                4 => warn!("Number of non existing packages which are needed as dependency: {}", count),
                5 => warn!("Number of non existing packages which are needed as dependency in renamed package: {}", count),
                6 => error!("Number of renamed packages that need renamed packages: {}", count),
                7 => error!("Number of renamed packages which are in component: {}", count),
                8 => error!("Number of obsoleted packages which are in component: {}", count),
                9 => error!("Number of obsoleted packages which are needed as dependency: {}", count),
                10 => error!("Number of obsoleted packages which are needed as dependency in renamed package: {}", count),
                11 => error!("Number of un-runnable make commands: {}", count),
                12 => error!("Number of packages that are in multiple components: {}", count),
                13 => error!("Number of packages that are in pkg5 file but do not exist: {}", count),
                14 => error!("Number of problems with packages that have same publisher: {}", count),
                _ => panic!("invalid problem type"),
            }
        }
    }

    pub fn get_problems_related_to_fmri(&self, fmri: &FMRI) -> Vec<Problem> {
        let mut problems: Vec<Problem> = Vec::new();
        for problem in self.get_ref() {
            match problem {
                UselessComponent(_) => {}
                UnRunnableMakeCommand(_, _) => {}
                NonExistingPackageInPkg5(f, _)
                | SamePackageHasTwoPublishers(f, _, _, _)
                | PackageInMultipleComponents(f, _)
                | MissingComponentForPackage(f)
                | RenamedPackageInComponent(f, _)
                | ObsoletedPackageInComponent(f, _) => {
                    if f.package_name_eq(fmri) {
                        problems.push(problem.clone());
                    }
                }
                RenamedNeedsRenamed(f_a, f_b) => {
                    if f_a.package_name_eq(fmri) || f_b.package_name_eq(fmri) {
                        problems.push(problem.clone());
                    }
                }
                NonExistingRequired(depend_type, _, f, _)
                | NonExistingRequiredByRenamed(depend_type, _, f)
                | ObsoletedRequired(depend_type, _, f, _)
                | ObsoletedRequiredByRenamed(depend_type, _, f)
                | PartlyObsoletedRequired(depend_type, _, f, _)
                | PartlyObsoletedRequiredByRenamed(depend_type, _, f) => {
                    if f.package_name_eq(fmri) {
                        problems.push(problem.clone());
                        continue;
                    }

                    match depend_type {
                        DependTypes::Require(f)
                        | DependTypes::Optional(f)
                        | DependTypes::Exclude(f)
                        | DependTypes::Incorporate(f)
                        | DependTypes::Origin(f)
                        | DependTypes::Group(f)
                        | DependTypes::Parent(f) => {
                            if f.package_name_eq(fmri) {
                                problems.push(problem.clone());
                            }
                        }
                        DependTypes::RequireAny(f_list) => {
                            if f_list.contains(fmri) {
                                problems.push(problem.clone());
                            }
                        }
                        DependTypes::GroupAny(f_list) => {
                            if f_list.contains(fmri) {
                                problems.push(problem.clone());
                            }
                        }
                        DependTypes::Conditional(f, predicate) => {
                            if f.package_name_eq(fmri) || predicate.package_name_eq(fmri) {
                                problems.push(problem.clone());
                            }
                        }
                    }
                }
            }
        }
        problems
    }
}

impl Default for Problems {
    fn default() -> Self {
        Self::new()
    }
}

pub fn report(problems: &Problems) {
    for problem in problems.get_ref() {
        report_problem(problem);
    }
    problems.count();
}

pub fn report_problem(problem: &Problem) {
    match problem {
        SamePackageHasTwoPublishers(fmri, publisher_a, publisher_b, p) => {
            if let Some(p) = p {
                error!(
                    "package {} has two publishers ({} and {}), but the latest version with publisher {} is not obsoleted",
                    fmri, publisher_a, publisher_b, p
                )
            } else {
                error!(
                    "package {} has two publishers ({} and {}), but wrong package is obsoleted",
                    fmri, publisher_a, publisher_b
                )
            }
        }
        NonExistingPackageInPkg5(fmri, component_name) => {
            error!(
                "package {} does not exist but it is in the pkg5, component: {}",
                fmri, component_name
            )
        }
        PackageInMultipleComponents(fmri, components) => {
            error!(
                "package {} is in multiple components: {}",
                fmri,
                components.join(",")
            )
        }
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

        NonExistingRequired(depend_type, dependency_type, required_by, component_name) => {
            let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

            let package_or_component_name = if dependency_type == &DependencyTypes::Runtime {
                required_by.get_package_name_as_ref_string().clone()
            } else {
                format!("component {}", component_name)
            };

            warn!(
                "package {} doesn't exist, but is required by {}",
                fmri,
                match dependency_type {
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
                }
            )
        }
        NonExistingRequiredByRenamed(depend_type, dependency_type, required_by) => {
            let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

            let package_name = required_by.get_package_name_as_ref_string();

            warn!(
                "package {} doesn't exist, but is required by renamed package {}",
                fmri,
                match dependency_type {
                    DependencyTypes::Runtime => {
                        format!("{} (runtime, {})", package_name, name)
                    }
                    DependencyTypes::Build => {
                        format!("{} (build, {})", package_name, name)
                    }
                    DependencyTypes::Test => {
                        format!("{} (test, {})", package_name, name)
                    }
                    DependencyTypes::SystemBuild => {
                        format!("{} (system-build)", package_name)
                    }
                    DependencyTypes::SystemTest => {
                        format!("{} (system-test)", package_name)
                    }
                }
            )
        }

        ObsoletedRequired(depend_type, dependency_type, required_by, component_name) => {
            let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

            let package_or_component_name = if dependency_type == &DependencyTypes::Runtime {
                required_by.get_package_name_as_ref_string().clone()
            } else {
                format!("component {}", component_name)
            };

            error!(
                "obsoleted package {} is required by {}",
                fmri,
                match dependency_type {
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
                }
            );
        }

        ObsoletedRequiredByRenamed(depend_type, dependency_type, required_by) => {
            let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

            let package_name = required_by.get_package_name_as_ref_string();

            error!(
                "obsoleted package {} is required by renamed package {}",
                fmri,
                match dependency_type {
                    DependencyTypes::Runtime => {
                        format!("{} (runtime, {})", package_name, name)
                    }
                    DependencyTypes::Build => {
                        format!("{} (build, component)", package_name)
                    }
                    DependencyTypes::Test => {
                        format!("{} (test, component)", package_name)
                    }
                    DependencyTypes::SystemBuild => {
                        format!("{} (system-build, system)", package_name)
                    }
                    DependencyTypes::SystemTest => {
                        format!("{} (system-test, system)", package_name)
                    }
                }
            );
        }

        PartlyObsoletedRequired(depend_type, dependency_type, required_by, component_name) => {
            let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

            let package_or_component_name = if dependency_type == &DependencyTypes::Runtime {
                required_by.get_package_name_as_ref_string().clone()
            } else {
                format!("component {}", component_name)
            };

            warn!(
                "obsoleted package {} is required by {}",
                fmri,
                match dependency_type {
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
                }
            );
        }
        PartlyObsoletedRequiredByRenamed(depend_type, dependency_type, required_by) => {
            let (name, fmri) = depend_type.clone().get_name_and_content_as_string();

            let package_name = required_by.get_package_name_as_ref_string();

            warn!(
                "obsoleted package {} is required by renamed package {}",
                fmri,
                match dependency_type {
                    DependencyTypes::Runtime => {
                        format!("{} (runtime, {})", package_name, name)
                    }
                    DependencyTypes::Build => {
                        format!("{} (build, component)", package_name)
                    }
                    DependencyTypes::Test => {
                        format!("{} (test, component)", package_name)
                    }
                    DependencyTypes::SystemBuild => {
                        format!("{} (system-build, system)", package_name)
                    }
                    DependencyTypes::SystemTest => {
                        format!("{} (system-test, system)", package_name)
                    }
                }
            );
        }
    }
}
