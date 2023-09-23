use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::AddAssign;
use serde::{Deserialize, Serialize};
use fmri::FMRI;
use fmri::fmri_list::FMRIList;
use fmri::Compare;
use crate::packages::components::Components;
use crate::packages::dependency::Dependency;
use crate::packages::depend_types::DependTypes;

/// Represents more [dependencies][Dependency]
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Dependencies(Vec<Dependency>);

impl Dependencies {
    /// Creates empty [`Dependencies`]
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Creates [`Dependencies`] from [`FMRIList`] with Require type
    pub fn new_from_fmri_list(fmri_list: FMRIList) -> Self {
        let mut dependencies = Self::new();

        for fmri in fmri_list.get() {
            dependencies.add(Dependency::new(&mut DependTypes::Require(fmri)))
        }

        dependencies
    }

    /// Adds [`Dependency`] into [`Dependencies`]
    pub fn add(&mut self, dependency: Dependency) {
        self.0.push(dependency)
    }

    /// Returns [`Vec`] of [dependencies][Dependency]
    pub fn get(self) -> Vec<Dependency> {
        self.0
    }

    /// Returns &[`Vec`] of [dependencies][Dependency]
    pub fn get_ref(&self) -> &Vec<Dependency> {
        &self.0
    }

    /// Checks if inserted [`FMRI`] is needed in [`self`] as [`Dependency`]
    pub fn is_fmri_needed_as_dependency(&self, components: &Components, checking_fmri: &FMRI) -> Option<Dependency> {
        for dependency in self.get_ref() {
            match dependency.get_ref() {
                DependTypes::Require(fmri) => {
                    if components.check_require_dependency(fmri, checking_fmri) {
                        return Some(dependency.clone());
                    }
                    // dependency is type require, but other conditions are not met
                }
                DependTypes::Optional(fmri) => {
                    if components.check_require_dependency(fmri, checking_fmri) {
                        return Some(dependency.clone());
                    }
                    // dependency is type optional, but other conditions are not met
                }
                DependTypes::Incorporate(fmri) => {
                    if fmri.package_name_eq(checking_fmri) {
                        match fmri.compare(checking_fmri) {
                            // incorporate need exact same version
                            Ordering::Equal => {
                                return Some(dependency.clone());
                            }
                            Ordering::Greater | Ordering::Less => {}
                        };
                    }
                    // dependency is type incorporate, but other conditions are not met
                }
                DependTypes::RequireAny(fmri_list) => {
                    for fmri in fmri_list.get_ref() {
                        if components.check_require_dependency(fmri, checking_fmri) {
                            return Some(dependency.clone());
                        }
                    }
                    // dependency is type require-any, but it is unneeded or other conditions are not met
                }
                DependTypes::Conditional(fmri, _) => {
                    if components.check_require_dependency(fmri, checking_fmri) {
                        return Some(dependency.clone());
                    }

                    // dependency is type conditional, but other conditions are not met
                }
                DependTypes::Group(fmri) => {
                    if components.check_require_dependency(fmri, checking_fmri) {
                        return Some(dependency.clone());
                    }
                    // dependency is type group, but other conditions are not met
                }
                _ => unimplemented!()
            };
        }

        // there aren't dependencies so it is not needed
        None
    }

    /// Returns true if [`self`] has inputted &[`Dependency`]
    pub fn contains(&self, dependency: &Dependency) -> bool {
        for self_dependency in self.get_ref() {
            if self_dependency == dependency {
                return true;
            }
        }
        false
    }
}

/// Implementation of [AddAssign] for [`Dependencies`]
impl AddAssign for Dependencies {
    /// Implements += operator for [`Dependencies`]
    ///
    /// Merges two [`Dependencies`], the exact same [`Dependencies`][Dependency] will be dumped
    ///
    /// # Examples
    ///
    /// _imagine [`dependencies`][Dependency] are numbers_
    /// ```plain
    /// [1, 2, 3] + [2, 3, 4] == [1, 2, 3, 4]
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        for rhs_dependency in rhs.get() {
            if !self.contains(&rhs_dependency) {
                self.add(rhs_dependency)
            }
        }
    }
}
