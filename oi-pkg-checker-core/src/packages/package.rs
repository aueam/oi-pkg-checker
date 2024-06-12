use std::cmp::Ordering;

use fmri::FMRI;
use serde::{Deserialize, Serialize};

use crate::packages::{components::Components, dependencies::Dependencies, dependency::Dependency};

/// Package contains dependencies
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Package {
    fmri: FMRI,
    obsolete: bool,
    renamed: bool,
    runtime: Dependencies,
    build: Dependencies,
    test: Dependencies,
    system_build: Dependencies,
    system_test: Dependencies,
}

impl Package {
    pub fn new(package_fmri: FMRI, obsolete: bool, renamed: bool) -> Self {
        Self {
            fmri: package_fmri,
            obsolete,
            renamed,
            runtime: Dependencies::new(),
            build: Dependencies::new(),
            test: Dependencies::new(),
            system_build: Dependencies::new(),
            system_test: Dependencies::new(),
        }
    }

    pub fn fmri_ref(&self) -> &FMRI {
        &self.fmri
    }

    pub fn fmri(self) -> FMRI {
        self.fmri
    }

    pub fn is_fmri_needed_as_dependency(
        &self,
        components: &Components,
        fmri: &FMRI,
    ) -> Option<Vec<(FMRI, String, Dependency)>> {
        // TODO: what to do if a package is dependent on itself?

        let mut required_dependencies: Vec<(FMRI, String, Dependency)> = Vec::new();

        if let Some(dependency) = self
            .get_runtime_dependencies_as_struct()
            .is_fmri_needed_as_dependency(components, fmri)
        {
            required_dependencies.push((self.clone().fmri(), "RUNTIME".to_owned(), dependency));
        }

        if let Some(dependency) = self
            .get_build_dependencies_as_struct()
            .is_fmri_needed_as_dependency(components, fmri)
        {
            required_dependencies.push((self.clone().fmri(), "BUILD".to_owned(), dependency));
        }

        if let Some(dependency) = self
            .get_test_dependencies_as_struct()
            .is_fmri_needed_as_dependency(components, fmri)
        {
            required_dependencies.push((self.clone().fmri(), "TEST".to_owned(), dependency));
        }

        if let Some(dependency) = self
            .get_system_build_dependencies_as_struct()
            .is_fmri_needed_as_dependency(components, fmri)
        {
            required_dependencies.push((
                self.clone().fmri(),
                "SYSTEM-BUILD".to_owned(),
                dependency,
            ));
        }

        if let Some(dependency) = self
            .get_system_test_dependencies_as_struct()
            .is_fmri_needed_as_dependency(components, fmri)
        {
            required_dependencies.push((self.clone().fmri(), "SYSTEM-TEST".to_owned(), dependency));
        }

        if required_dependencies.is_empty() {
            return None;
        }
        Some(required_dependencies)
    }

    pub fn is_obsolete(&self) -> bool {
        self.obsolete
    }

    pub fn is_renamed(&self) -> bool {
        self.renamed
    }

    pub fn get_runtime_dependencies_as_struct(&self) -> &Dependencies {
        &self.runtime
    }

    pub fn get_build_dependencies_as_struct(&self) -> &Dependencies {
        &self.build
    }

    pub fn get_test_dependencies_as_struct(&self) -> &Dependencies {
        &self.test
    }

    pub fn get_system_build_dependencies_as_struct(&self) -> &Dependencies {
        &self.system_build
    }

    pub fn get_system_test_dependencies_as_struct(&self) -> &Dependencies {
        &self.system_test
    }

    pub fn get_runtime_dependencies(&self) -> &Vec<Dependency> {
        self.runtime.get_ref()
    }

    pub fn get_build_dependencies(&self) -> &Vec<Dependency> {
        self.build.get_ref()
    }

    pub fn get_test_dependencies(&self) -> &Vec<Dependency> {
        self.test.get_ref()
    }

    pub fn get_system_build_dependencies(&self) -> &Vec<Dependency> {
        self.system_build.get_ref()
    }

    pub fn get_system_test_dependencies(&self) -> &Vec<Dependency> {
        self.system_test.get_ref()
    }

    pub fn add_runtime_dependencies(&mut self, dependencies: Dependencies) {
        self.runtime += dependencies
    }

    pub fn add_build_dependencies(&mut self, dependencies: Dependencies) {
        self.build += dependencies
    }

    pub fn add_test_dependencies(&mut self, dependencies: Dependencies) {
        self.test += dependencies
    }

    pub fn add_system_build_dependencies(&mut self, dependencies: Dependencies) {
        self.system_build += dependencies
    }

    pub fn add_system_test_dependencies(&mut self, dependencies: Dependencies) {
        self.system_test += dependencies
    }
}

impl PartialOrd<Self> for Package {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Package {
    /// Compares versions of FMRI
    fn cmp(&self, other: &Self) -> Ordering {
        self.fmri.cmp(&other.fmri)
    }
}
