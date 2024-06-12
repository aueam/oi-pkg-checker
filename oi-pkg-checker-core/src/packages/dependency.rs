use fmri::{FMRI, fmri_list::FMRIList};
use serde::{Deserialize, Serialize};

use crate::{
    packages::{
        components::Components, depend_types::DependTypes, dependency_type::DependencyTypes,
        package::Package,
    },
    Problems,
    problems::Problem::{
        NonExistingRequiredPackage, ObsoletedRequiredPackage, PartlyObsoletedRequiredPackage,
    },
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
        problems: &mut Problems,
        package: Package,
        dependency_type: DependencyTypes,
    ) {
        match self.get_ref() {
            DependTypes::Require(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        problems.add_problem(ObsoletedRequiredPackage(
                            DependTypes::Require(fmri.clone()),
                            dependency_type,
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    } else {
                        problems.add_problem(PartlyObsoletedRequiredPackage(
                            DependTypes::Require(fmri.clone()),
                            dependency_type,
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    problems.add_problem(NonExistingRequiredPackage(
                        DependTypes::Require(fmri.clone()),
                        dependency_type,
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }
            }
            DependTypes::Optional(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        problems.add_problem(ObsoletedRequiredPackage(
                            DependTypes::Optional(fmri.clone()),
                            dependency_type,
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    } else {
                        problems.add_problem(PartlyObsoletedRequiredPackage(
                            DependTypes::Optional(fmri.clone()),
                            dependency_type,
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    problems.add_problem(NonExistingRequiredPackage(
                        DependTypes::Optional(fmri.clone()),
                        dependency_type,
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }
            }
            DependTypes::Incorporate(fmri) => {
                if !components.is_fmri_obsoleted(fmri)
                    && !components.check_if_fmri_exists_as_package(fmri)
                {
                    problems.add_problem(NonExistingRequiredPackage(
                        DependTypes::Incorporate(fmri.clone()),
                        dependency_type,
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }
            }
            DependTypes::RequireAny(fmri_list) => {
                let mut fmri_list_obsolete = FMRIList::new();
                let mut fmri_list_partly_obsolete = FMRIList::new();
                let mut fmri_list_non_existing = FMRIList::new();

                for fmri in fmri_list.get_ref() {
                    if components.is_fmri_obsoleted(fmri) {
                        if !components.check_if_fmri_exists_as_package(fmri) {
                            fmri_list_obsolete.add(fmri.clone());
                        } else {
                            fmri_list_partly_obsolete.add(fmri.clone());
                        }
                    } else if !components.check_if_fmri_exists_as_package(fmri) {
                        fmri_list_non_existing.add(fmri.clone())
                    }
                }

                if !fmri_list_obsolete.is_empty() {
                    problems.add_problem(ObsoletedRequiredPackage(
                        DependTypes::RequireAny(fmri_list_obsolete),
                        dependency_type.clone(),
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }

                if !fmri_list_partly_obsolete.is_empty() {
                    problems.add_problem(PartlyObsoletedRequiredPackage(
                        DependTypes::RequireAny(fmri_list_partly_obsolete),
                        dependency_type.clone(),
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }

                if !fmri_list_non_existing.is_empty() {
                    problems.add_problem(NonExistingRequiredPackage(
                        DependTypes::RequireAny(fmri_list_non_existing),
                        dependency_type,
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }
            }
            DependTypes::Conditional(fmri, predicate) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        problems.add_problem(ObsoletedRequiredPackage(
                            DependTypes::Conditional(
                                fmri.clone(),
                                FMRI::parse_raw("none").unwrap(),
                            ),
                            dependency_type.clone(),
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    } else {
                        problems.add_problem(PartlyObsoletedRequiredPackage(
                            DependTypes::Conditional(
                                fmri.clone(),
                                FMRI::parse_raw("none").unwrap(),
                            ),
                            dependency_type.clone(),
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    problems.add_problem(NonExistingRequiredPackage(
                        DependTypes::Conditional(fmri.clone(), FMRI::parse_raw("none").unwrap()),
                        dependency_type.clone(),
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }

                if components.is_fmri_obsoleted(predicate) {
                    if !components.check_if_fmri_exists_as_package(predicate) {
                        problems.add_problem(ObsoletedRequiredPackage(
                            DependTypes::Conditional(
                                FMRI::parse_raw("none").unwrap(),
                                predicate.clone(),
                            ),
                            dependency_type.clone(),
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    } else {
                        problems.add_problem(PartlyObsoletedRequiredPackage(
                            DependTypes::Conditional(
                                FMRI::parse_raw("none").unwrap(),
                                predicate.clone(),
                            ),
                            dependency_type.clone(),
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    }
                } else if !components.check_if_fmri_exists_as_package(predicate) {
                    problems.add_problem(NonExistingRequiredPackage(
                        DependTypes::Conditional(
                            FMRI::parse_raw("none").unwrap(),
                            predicate.clone(),
                        ),
                        dependency_type.clone(),
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }
            }
            DependTypes::Group(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        problems.add_problem(ObsoletedRequiredPackage(
                            DependTypes::Group(fmri.clone()),
                            dependency_type,
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    } else {
                        problems.add_problem(PartlyObsoletedRequiredPackage(
                            DependTypes::Group(fmri.clone()),
                            dependency_type.clone(),
                            package.clone().fmri(),
                            package.is_renamed(),
                        ));
                    }
                } else if !components.check_if_fmri_exists_as_package(fmri) {
                    problems.add_problem(NonExistingRequiredPackage(
                        DependTypes::Group(fmri.clone()),
                        dependency_type.clone(),
                        package.clone().fmri(),
                        package.is_renamed(),
                    ));
                }
            }
            _ => unimplemented!(),
        }
    }
}
