use serde::{Deserialize, Serialize};

use crate::packages::{
    dependency::Dependency, dependency_type::DependencyTypes, package_versions::PackageVersions,
};

/// Component has 1 or more packages
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Component {
    name: String,
    package_versions: Vec<PackageVersions>, // TODO: it should be Vec<&PackageVersions>
}

impl Component {
    pub fn new(name: String) -> Self {
        Self {
            name,
            package_versions: vec![],
        }
    }

    pub fn add(&mut self, packages: PackageVersions) {
        self.package_versions.push(packages)
    }

    pub fn get_versions(self) -> Vec<PackageVersions> {
        self.package_versions
    }

    pub fn get_versions_ref(&self) -> &Vec<PackageVersions> {
        &self.package_versions
    }

    pub fn get_versions_ref_mut(&mut self) -> &mut Vec<PackageVersions> {
        &mut self.package_versions
    }

    pub fn get_dependencies(
        &self,
        dependency_types: Vec<DependencyTypes>,
    ) -> Vec<(Dependency, DependencyTypes)> {
        let package = self
            .get_versions_ref()
            .last()
            .expect("empty component")
            .get_packages_ref()
            .last()
            .expect("empty package versions");

        let mut dependencies: Vec<(Dependency, DependencyTypes)> = Vec::new();

        for dependency_type in dependency_types {
            match dependency_type {
                DependencyTypes::Runtime => {
                    for dependency in package.get_runtime_dependencies().clone() {
                        let mut found = false;
                        for (from_dependency, _) in &dependencies {
                            if from_dependency == &dependency {
                                found = true
                            }
                        }
                        if !found {
                            dependencies.push((dependency, DependencyTypes::Runtime))
                        }
                    }
                }
                DependencyTypes::Build => {
                    for dependency in package.get_build_dependencies().clone() {
                        let mut found = false;
                        for (from_dependency, _) in &dependencies {
                            if from_dependency == &dependency {
                                found = true
                            }
                        }
                        if !found {
                            dependencies.push((dependency, DependencyTypes::Build))
                        }
                    }
                }
                DependencyTypes::Test => {
                    for dependency in package.get_test_dependencies().clone() {
                        let mut found = false;
                        for (from_dependency, _) in &dependencies {
                            if from_dependency == &dependency {
                                found = true
                            }
                        }
                        if !found {
                            dependencies.push((dependency, DependencyTypes::Test))
                        }
                    }
                }
                DependencyTypes::SystemBuild => {
                    for dependency in package.get_system_build_dependencies().clone() {
                        let mut found = false;
                        for (from_dependency, _) in &dependencies {
                            if from_dependency == &dependency {
                                found = true
                            }
                        }
                        if !found {
                            dependencies.push((dependency, DependencyTypes::SystemBuild))
                        }
                    }
                }
                DependencyTypes::SystemTest => {
                    for dependency in package.get_system_test_dependencies().clone() {
                        let mut found = false;
                        for (from_dependency, _) in &dependencies {
                            if from_dependency == &dependency {
                                found = true
                            }
                        }
                        if !found {
                            dependencies.push((dependency, DependencyTypes::SystemTest))
                        }
                    }
                }
                DependencyTypes::None => unimplemented!(),
            };
        }

        dependencies
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name
    }

    pub fn get_name(self) -> String {
        self.name
    }

    pub fn get_name_ref(&self) -> &String {
        &self.name
    }

    pub fn change_versions(&mut self, package_versions: Vec<PackageVersions>) {
        self.package_versions = package_versions
    }
}
