use std::cell::RefCell;

use fmri::{fmri_list::FMRIList, FMRI};
use serde::{Deserialize, Serialize};

use crate::{
    packages::{
        components::Components, depend_types::DependTypes, dependency_type::DependencyTypes,
        package::Package,
    },
    problems::Problem::{
        NonExistingRequired, NonExistingRequiredByRenamed, ObsoletedRequired,
        ObsoletedRequiredByRenamed, PartlyObsoletedRequired, PartlyObsoletedRequiredByRenamed,
    },
    Problems,
};

/// Represents depend action, it contains [`DependTypes`], all [`FMRIs`][`FMRI`] in it are without [`Publisher`]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Dependency(DependTypes);

impl Dependency {
    /// Creates new [`Dependency`]
    pub fn new(d_type: &DependTypes) -> Self {
        let d_type = &mut d_type.clone();
        match d_type {
            DependTypes::Require(fmri) => fmri.remove_publisher(),
            DependTypes::Optional(fmri) => fmri.remove_publisher(),
            DependTypes::Incorporate(fmri) => fmri.remove_publisher(),
            DependTypes::RequireAny(fmri_list) => {
                for fmri in fmri_list.get_ref_mut() {
                    fmri.remove_publisher()
                }
            }
            DependTypes::Conditional(fmri, predicate) => {
                fmri.remove_publisher();
                predicate.remove_publisher();
            }
            DependTypes::Group(fmri) => fmri.remove_publisher(),
            _ => unimplemented!(),
        }

        Self(d_type.clone())
    }

    /// Returns [`DependTypes`]
    pub fn get(self) -> DependTypes {
        self.0
    }

    /// Returns &[`DependTypes`]
    pub fn get_ref(&self) -> &DependTypes {
        &self.0
    }

    /// Returns &mut [`DependTypes`]
    pub fn get_ref_mut(&mut self) -> &mut DependTypes {
        &mut self.0
    }

    /// This function returns problems with dependency
    pub fn check_dependency_validity(
        &self,
        components: &Components,
        mut problems: &mut Problems,
        package: Package,
        dependency_type: DependencyTypes,
    ) {
        let problems = RefCell::new(&mut problems);

        let obsoleted = |d_type: DependTypes| {
            let package_fmri = package.clone().fmri();

            problems.borrow_mut().add_problem(if package.is_renamed() {
                ObsoletedRequiredByRenamed(d_type, dependency_type.clone(), package_fmri)
            } else {
                let component_name = components
                    .get_component_name_by_package(&package_fmri)
                    .unwrap()
                    .clone();
                ObsoletedRequired(
                    d_type,
                    dependency_type.clone(),
                    package_fmri,
                    component_name,
                )
            });
        };

        let partly = |d_type: DependTypes| {
            let package_fmri = package.clone().fmri();

            problems.borrow_mut().add_problem(if package.is_renamed() {
                PartlyObsoletedRequiredByRenamed(d_type, dependency_type.clone(), package_fmri)
            } else {
                let component_name = components
                    .get_component_name_by_package(&package_fmri)
                    .unwrap()
                    .clone();
                PartlyObsoletedRequired(
                    d_type,
                    dependency_type.clone(),
                    package_fmri,
                    component_name,
                )
            });
        };

        let non_existing = |d_type: DependTypes| {
            let package_fmri = package.clone().fmri();

            problems.borrow_mut().add_problem(if package.is_renamed() {
                NonExistingRequiredByRenamed(d_type, dependency_type.clone(), package_fmri)
            } else {
                let component_name = components
                    .get_component_name_by_package(&package_fmri)
                    .unwrap()
                    .clone();
                NonExistingRequired(
                    d_type,
                    dependency_type.clone(),
                    package_fmri,
                    component_name,
                )
            });
        };

        match self.get_ref() {
            DependTypes::Require(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsoleted(DependTypes::Require(fmri.clone()));
                    } else {
                        partly(DependTypes::Require(fmri.clone()));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    non_existing(DependTypes::Require(fmri.clone()));
                }
            }
            DependTypes::Optional(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsoleted(DependTypes::Optional(fmri.clone()));
                    } else {
                        partly(DependTypes::Optional(fmri.clone()));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    non_existing(DependTypes::Optional(fmri.clone()));
                }
            }
            DependTypes::Incorporate(fmri) => {
                if !components.is_fmri_obsoleted(fmri)
                    && !components.check_if_fmri_exists_as_package(fmri)
                {
                    non_existing(DependTypes::Incorporate(fmri.clone()));
                }
            }
            DependTypes::RequireAny(fmri_list) => {
                let mut obsolete_list = FMRIList::new();
                let mut partly_obsolete_list = FMRIList::new();
                let mut non_existing_list = FMRIList::new();

                for fmri in fmri_list.get_ref() {
                    if components.is_fmri_obsoleted(fmri) {
                        if !components.check_if_fmri_exists_as_package(fmri) {
                            obsolete_list.add(fmri.clone());
                        } else {
                            partly_obsolete_list.add(fmri.clone());
                        }
                    } else if !components.check_if_fmri_exists_as_package(fmri) {
                        non_existing_list.add(fmri.clone())
                    }
                }

                if !obsolete_list.is_empty() {
                    obsoleted(DependTypes::RequireAny(obsolete_list));
                }

                if !partly_obsolete_list.is_empty() {
                    partly(DependTypes::RequireAny(partly_obsolete_list));
                }

                if !non_existing_list.is_empty() {
                    non_existing(DependTypes::RequireAny(non_existing_list));
                }
            }
            DependTypes::Conditional(fmri, predicate) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsoleted(DependTypes::Conditional(
                            fmri.clone(),
                            FMRI::parse_raw("none").unwrap(),
                        ));
                    } else {
                        partly(DependTypes::Conditional(
                            fmri.clone(),
                            FMRI::parse_raw("none").unwrap(),
                        ));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    non_existing(DependTypes::Conditional(
                        fmri.clone(),
                        FMRI::parse_raw("none").unwrap(),
                    ));
                }

                if components.is_fmri_obsoleted(predicate) {
                    if !components.check_if_fmri_exists_as_package(predicate) {
                        obsoleted(DependTypes::Conditional(
                            FMRI::parse_raw("none").unwrap(),
                            fmri.clone(),
                        ));
                    } else {
                        partly(DependTypes::Conditional(
                            FMRI::parse_raw("none").unwrap(),
                            fmri.clone(),
                        ));
                    }
                } else if !components.check_if_fmri_exists_as_package(predicate) {
                    non_existing(DependTypes::Conditional(
                        FMRI::parse_raw("none").unwrap(),
                        fmri.clone(),
                    ));
                }
            }
            DependTypes::Group(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsoleted(DependTypes::Group(fmri.clone()));
                    } else {
                        partly(DependTypes::Group(fmri.clone()));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    non_existing(DependTypes::Group(fmri.clone()));
                }
            }
            _ => unimplemented!(),
        }
    }
}
