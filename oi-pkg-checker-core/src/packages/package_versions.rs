use crate::packages::package::Package;
use fmri::{compare::Compare, FMRI};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// PackageVersions has 1 or more versions of package
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct PackageVersions {
    /// only publisher and package_name, no version! (version doesn't make sense here)
    pub(crate) fmri: FMRI,
    /// obsolete means that package is obsolete or renamed
    pub(crate) obsolete: bool,
    /// obsolete means that package is obsolete or renamed
    pub(crate) renamed: bool,
    /// version, package
    pub(crate) packages: Vec<Package>,
}

impl PackageVersions {
    pub fn new(mut fmri: FMRI) -> Self {
        if fmri.has_version() {
            fmri.remove_version();
        }

        Self {
            fmri,
            obsolete: false,
            renamed: false,
            packages: vec![],
        }
    }

    /// Adds [`Package`] to [`PackageVersions`] with some rules:
    /// return if there is already newer one in [`PackageVersions`]
    /// if adding [`Package`] is obsolete, add obsolete flag and return (if not, remove it)
    /// if adding [`Package`] is renamed, add renamed flag (if not, remove it)
    /// if there is already older [`Package`], remove it
    /// and now the new [`Package`]  can be added
    pub fn add_package(&mut self, package: Package) -> Option<()> {
        if package.is_obsolete() && package.is_renamed() {
            panic!(
                "package cannot be obsolete and renamed at the same time, package: {:?}",
                package
            )
        }

        if let Some(newer_package) = &self.get_newer_package() {
            match package.compare(newer_package) {
                Ordering::Less => todo!("package is older than newer one"),
                Ordering::Greater | Ordering::Equal => {}
            }
        }

        if package.is_obsolete() {
            self.set_obsolete(true);
            return Some(());
        } else {
            self.set_obsolete(false);
        }

        if package.is_renamed() {
            self.set_renamed(true);
        } else {
            self.set_renamed(false);
        }

        if let Some(newer_package) = &self.get_newer_package() {
            self.remove_package(newer_package)
        }

        self.packages.push(package);
        None
    }

    pub fn fmri(self) -> FMRI {
        self.fmri
    }

    pub fn fmri_ref(&self) -> &FMRI {
        &self.fmri
    }

    pub fn get_packages(self) -> Vec<Package> {
        self.packages
    }

    pub fn get_packages_ref(&self) -> &Vec<Package> {
        &self.packages
    }

    pub fn get_packages_ref_mut(&mut self) -> &mut Vec<Package> {
        &mut self.packages
    }

    pub fn set_obsolete(&mut self, obsolete: bool) {
        self.obsolete = obsolete
    }

    pub fn set_renamed(&mut self, renamed: bool) {
        self.renamed = renamed
    }

    pub fn is_obsolete(&self) -> bool {
        self.obsolete
    }

    pub fn is_renamed(&self) -> bool {
        self.renamed
    }

    /// Returns newer [`Package`] in [`PackageVersions`] if there is at least one
    pub fn get_newer_package(&self) -> Option<Package> {
        let packages = self.get_packages_ref();
        if packages.is_empty() {
            return None;
        }

        let mut newer_package = packages[0].clone();
        for package in packages {
            match newer_package.compare(package) {
                Ordering::Less => newer_package = package.clone(),
                Ordering::Equal => {}
                Ordering::Greater => {}
            }
        }

        Some(newer_package)
    }

    /// Removes [`Package`] from [`PackageVersions`]
    pub fn remove_package(&mut self, package_to_remove: &Package) {
        let packages = self.get_packages_ref_mut();
        if packages.is_empty() {
            panic!("No package to remove, but at least one was expected to be there")
        }

        for (index, package) in packages.iter().enumerate() {
            match package.compare(package_to_remove) {
                // finding exact same package to remove
                Ordering::Equal => {
                    packages.remove(index);
                    return;
                }
                Ordering::Greater | Ordering::Less => {}
            }
        }
    }
}
