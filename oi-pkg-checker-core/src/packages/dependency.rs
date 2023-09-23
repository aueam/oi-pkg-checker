use serde::{Deserialize, Serialize};
use fmri::FMRI;
use fmri::fmri_list::FMRIList;
use crate::packages::components::Components;
use crate::packages::depend_types::DependTypes;
use crate::packages::dependency_type::DependencyTypes;
use crate::packages::package::Package;
use crate::problems::{NonExistingRequiredPackage, NonExistingRequiredPackageList, ObsoletedRequiredPackage, ObsoletedRequiredPackageList, PartlyObsoletedRequiredPackage, PartlyObsoletedRequiredPackageList, ProblemList};

/// Represents depend action, it contains [`DependTypes`], all [`FMRIs`][`FMRI`] in it are without [`Publisher`]
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
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
            _ => unimplemented!()
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
        package: Package,
    ) -> Result<(), (NonExistingRequiredPackageList, ObsoletedRequiredPackageList, PartlyObsoletedRequiredPackageList)> {
        let mut obsolete_required_package_list = ObsoletedRequiredPackageList::new();
        let mut non_existing_required_package_list = NonExistingRequiredPackageList::new();
        let mut partly_obsolete_required_package_list = PartlyObsoletedRequiredPackageList::new();

        match self.get_ref() {
            DependTypes::Require(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsolete_required_package_list.add(
                            ObsoletedRequiredPackage::new(DependTypes::Require(fmri.clone()), DependencyTypes::None, package.clone().fmri(), package.is_renamed())
                        );
                    } else {
                        partly_obsolete_required_package_list.add(
                            PartlyObsoletedRequiredPackage::new(DependTypes::Require(fmri.clone()), DependencyTypes::None, package.clone().fmri(), package.is_renamed())
                        );
                    }
                } else {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        non_existing_required_package_list.add(
                            NonExistingRequiredPackage::new(DependTypes::Require(fmri.clone()), DependencyTypes::None, package.clone().fmri(), package.is_renamed())
                        );
                    }
                }
            }
            DependTypes::Optional(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsolete_required_package_list.add(
                            ObsoletedRequiredPackage::new(DependTypes::Optional(fmri.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    } else {
                        partly_obsolete_required_package_list.add(
                            PartlyObsoletedRequiredPackage::new(DependTypes::Optional(fmri.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                } else {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        non_existing_required_package_list.add(
                            NonExistingRequiredPackage::new(DependTypes::Optional(fmri.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                }
            }
            DependTypes::Incorporate(fmri) => {
                if !components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        non_existing_required_package_list.add(
                            NonExistingRequiredPackage::new(DependTypes::Incorporate(fmri.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
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
                    } else {
                        if !components.check_if_fmri_exists_as_package(fmri) {
                            fmri_list_non_existing.add(fmri.clone())
                        }
                    }
                }

                if fmri_list_obsolete.len() != 0 {
                    obsolete_required_package_list.add(
                        ObsoletedRequiredPackage::new(DependTypes::RequireAny(fmri_list_obsolete), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                    );
                }

                if fmri_list_partly_obsolete.len() != 0 {
                    partly_obsolete_required_package_list.add(
                        PartlyObsoletedRequiredPackage::new(DependTypes::RequireAny(fmri_list_partly_obsolete), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                    );
                }

                if fmri_list_non_existing.len() != 0 {
                    non_existing_required_package_list.add(
                        NonExistingRequiredPackage::new(DependTypes::RequireAny(fmri_list_non_existing), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                    );
                }
            }
            DependTypes::Conditional(fmri, predicate) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsolete_required_package_list.add(
                            ObsoletedRequiredPackage::new(DependTypes::Conditional(fmri.clone(), FMRI::parse_raw(&"none".to_owned())), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    } else {
                        partly_obsolete_required_package_list.add(
                            PartlyObsoletedRequiredPackage::new(DependTypes::Conditional(fmri.clone(), FMRI::parse_raw(&"none".to_owned())), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                } else {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        non_existing_required_package_list.add(
                            NonExistingRequiredPackage::new(DependTypes::Conditional(fmri.clone(), FMRI::parse_raw(&"none".to_owned())), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                }


                if components.is_fmri_obsoleted(predicate) {
                    if !components.check_if_fmri_exists_as_package(predicate) {
                        obsolete_required_package_list.add(
                            ObsoletedRequiredPackage::new(DependTypes::Conditional(FMRI::parse_raw(&"none".to_owned()), predicate.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    } else {
                        partly_obsolete_required_package_list.add(
                            PartlyObsoletedRequiredPackage::new(DependTypes::Conditional(FMRI::parse_raw(&"none".to_owned()), predicate.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                } else {
                    if !components.check_if_fmri_exists_as_package(predicate) {
                        non_existing_required_package_list.add(
                            NonExistingRequiredPackage::new(DependTypes::Conditional(FMRI::parse_raw(&"none".to_owned()), predicate.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                }
            }
            DependTypes::Group(fmri) => {
                if components.is_fmri_obsoleted(fmri) {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        obsolete_required_package_list.add(
                            ObsoletedRequiredPackage::new(DependTypes::Group(fmri.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    } else {
                        partly_obsolete_required_package_list.add(
                            PartlyObsoletedRequiredPackage::new(DependTypes::Group(fmri.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                } else {
                    if !components.check_if_fmri_exists_as_package(fmri) {
                        non_existing_required_package_list.add(
                            NonExistingRequiredPackage::new(DependTypes::Group(fmri.clone()), DependencyTypes::Runtime, package.clone().fmri(), package.is_renamed())
                        );
                    }
                }
            }
            _ => unimplemented!()
        }

        if !non_existing_required_package_list.is_empty() || !obsolete_required_package_list.is_empty() || !partly_obsolete_required_package_list.is_empty() {
            return Err((non_existing_required_package_list, obsolete_required_package_list, partly_obsolete_required_package_list));
        }

        Ok(())
    }
}