use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    fs::File,
    io::{Read, Write},
    path::Path,
};

use bincode::{deserialize, serialize};
use fmri::{FMRI, fmri_list::FMRIList};
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::{
    assets::{
        assets_types::AssetTypes,
        catalogs_c::load_catalog_c,
        open_indiana_oi_userland_git::{component_list, ComponentPackagesList, load_dependencies},
    },
    packages::{
        component::Component, dependency::Dependency, dependency_type::DependencyTypes,
        package_versions::PackageVersions,
    },
    Problems,
    problems::Problem::{RenamedNeedsRenamed, UselessComponent},
};

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Components {
    components: Vec<Component>,
    obsolete: FMRIList,
}

impl Components {
    pub fn new() -> Self {
        Self {
            components: vec![],
            obsolete: FMRIList::new(),
        }
    }

    pub fn load(
        &mut self,
        problems: &mut Problems,
        asset: AssetTypes,
        oi_userland_components: &Path,
    ) {
        let component_packages_list = ComponentPackagesList::new(oi_userland_components);

        match asset {
            AssetTypes::Catalogs(paths) => {
                for path in paths {
                    load_catalog_c(self, path, problems, &component_packages_list);
                }
            }
            AssetTypes::OpenIndianaOiUserlandGit => {
                component_list(self, problems, &component_packages_list);
                load_dependencies(
                    self,
                    problems,
                    &component_packages_list,
                    &DependencyTypes::Build,
                );
                load_dependencies(
                    self,
                    problems,
                    &component_packages_list,
                    &DependencyTypes::Test,
                );
                load_dependencies(
                    self,
                    problems,
                    &component_packages_list,
                    &DependencyTypes::SystemBuild,
                );
                load_dependencies(
                    self,
                    problems,
                    &component_packages_list,
                    &DependencyTypes::SystemTest,
                );
            }
        }
    }

    pub fn serialize<P: AsRef<Path> + ?Sized>(&self, path: &P) {
        File::create(path)
            .unwrap()
            .write_all(&serialize(self).expect("failed to serialize data into binary"))
            .expect("TODO: panic message");
    }

    pub fn deserialize<P: AsRef<Path> + ?Sized>(path: &P) -> Self {
        let data = &mut Vec::new();
        File::open(path)
            .expect("failed to open data file")
            .read_to_end(data)
            .expect("failed to read data");
        deserialize(data).expect("failed to deserialize data from binary")
    }

    pub fn add_package_to_component_with_name(
        &mut self,
        package_versions: &PackageVersions,
        component_name: String,
    ) {
        if !component_name.is_empty() {
            for component in self.get_ref_mut() {
                if component.get_name_ref() == &component_name {
                    debug!(
                        "adding {} into component: {}",
                        package_versions.fmri_ref(),
                        component_name
                    );
                    component.add(package_versions.clone());
                    return;
                }
            }
        }

        let mut new_component = Component::new(component_name);
        new_component.add(package_versions.clone());
        self.add(new_component);
    }

    pub fn name_unnamed_components(&mut self) {
        for component in self.get_ref_mut() {
            if component.get_name_ref() == "" {
                let package_versions = component.get_versions_ref();
                assert_eq!(package_versions.len(), 1);
                let name = "/".to_owned()
                    + &*package_versions
                        .first()
                        .unwrap()
                        .fmri_ref()
                        .get_package_name_as_ref_string()
                        .clone();
                debug!("naming unnamed component with name: /{}", &name);
                component.change_name(name);
            }
        }
    }

    pub fn get_useless_components(&self, problems: &mut Problems) {
        for component in self.get_ref() {
            if component.get_name_ref() == "" {
                continue;
            }

            let mut number_of_package_versions = component.get_versions_ref().len();

            for package_version in component.get_versions_ref() {
                if !self.is_fmri_required_dependency(package_version.fmri_ref()) {
                    number_of_package_versions -= 1;
                }
            }

            if number_of_package_versions == 0 {
                problems.add_problem(UselessComponent(component.clone().get_name()));
            }
        }
    }

    pub fn get_component_name_by_package(&self, package: &FMRI) -> Option<&String> {
        for component in self.get_ref() {
            for package_versions in component.get_versions_ref() {
                if package_versions.fmri_ref().package_name_eq(package) {
                    return Some(component.get_name_ref());
                }
            }
        }

        info!("can't find package {} it is maybe obsolete", package);
        None
    }

    // TODO: remake
    // pub fn check_component_cycles(&self, components_path: &PathBuf, dependency_types: Vec<DependencyTypes>) -> Option<Vec<CycleRoute>> {
    //     let counter: f32 = self.get_ref().len() as f32 / 100.;
    //     let mut last: i32 = 0;
    //     let cycle_routes = &mut Cycles::new();
    //     let was_there = &mut vec![];
    //     for (index, component) in self.get_ref().iter().enumerate() {
    //         component.find_cycles(
    //             cycle_routes,
    //             was_there,
    //             CycleRoute::new_empty(),
    //             self,
    //             dependency_types.clone(),
    //         );
    //
    //         let update = index as f32 / counter;
    //         if update as i32 > last {
    //             last = update as i32;
    //             info!("{}%", last);
    //         }
    //     }
    //
    //     info!("{}", was_there.len());
    //
    //     sleep(Duration::from_secs(5));
    //
    //     let mut cycles = cycle_routes.clone().get();
    //
    //     cycles.sort();
    //     cycles.dedup();
    //
    //     if cycles.len() == 0 {
    //         return None;
    //     }
    //     Some(cycles)
    // }

    pub fn is_fmri_required_dependency(&self, fmri: &FMRI) -> bool {
        for component in self.get_ref() {
            for package_version in component.get_versions_ref() {
                for package in package_version.get_packages_ref() {
                    if !package.fmri_ref().package_name_eq(fmri) {
                        match package.is_fmri_needed_as_dependency(self, fmri) {
                            None => {}
                            Some(_) => return true,
                        }
                    }
                }
            }
        }
        false
    }

    pub fn get_dependencies_with_fmri(
        &self,
        fmri: &FMRI,
    ) -> Option<Vec<(FMRI, String, Dependency, bool)>> {
        let mut list: Vec<(FMRI, String, Dependency, bool)> = Vec::new();
        for component in self.get_ref() {
            for package_version in component.get_versions_ref() {
                for package in package_version.get_packages_ref() {
                    if !package.fmri_ref().package_name_eq(fmri) {
                        if let Some(dependencies) = package.is_fmri_needed_as_dependency(self, fmri)
                        {
                            for (fmri, d_type, dependency, renamed) in dependencies {
                                list.push((fmri, d_type, dependency, renamed))
                            }
                        }
                    }
                }
            }
        }

        if list.is_empty() {
            return None;
        }
        Some(list)
    }

    pub fn check_dependency_validity(&self, problems: &mut Problems) {
        for component in self.get_ref() {
            for package_version in component.get_versions_ref() {
                for package in package_version.get_packages_ref() {
                    if package.is_obsolete() {
                        panic!("package can't be obsolete")
                    }

                    for runtime in package.get_runtime_dependencies() {
                        runtime.check_dependency_validity(
                            self,
                            problems,
                            package.clone(),
                            DependencyTypes::Runtime,
                        )
                    }

                    for build in package.get_build_dependencies() {
                        build.check_dependency_validity(
                            self,
                            problems,
                            package.clone(),
                            DependencyTypes::Build,
                        )
                    }

                    for test in package.get_test_dependencies() {
                        test.check_dependency_validity(
                            self,
                            problems,
                            package.clone(),
                            DependencyTypes::Test,
                        )
                    }

                    for system_build in package.get_system_build_dependencies() {
                        system_build.check_dependency_validity(
                            self,
                            problems,
                            package.clone(),
                            DependencyTypes::SystemBuild,
                        )
                    }

                    for system_test in package.get_system_test_dependencies() {
                        system_test.check_dependency_validity(
                            self,
                            problems,
                            package.clone(),
                            DependencyTypes::SystemTest,
                        )
                    }
                }
            }
        }
    }

    pub fn remove_empty_components(&mut self) {
        let mut components: Vec<Component> = vec![];
        for component in self.clone().get() {
            if !component.clone().get_versions().is_empty() {
                components.push(component)
            }
        }
        self.components = components
    }

    pub fn remove_empty_package_versions(&mut self) {
        for component in self.get_ref_mut() {
            let mut new_component: Vec<PackageVersions> = vec![];
            for package_version in component.clone().get_versions() {
                if !package_version.get_packages_ref().is_empty() {
                    new_component.push(package_version)
                }
            }
            component.change_versions(new_component)
        }
    }

    pub fn is_there_newer_version(&self, fmri: &FMRI) -> Option<FMRI> {
        for component in self.get_ref() {
            for package_version in component.get_versions_ref() {
                if package_version.fmri_ref().package_name_eq(fmri) {
                    for package in package_version.get_packages_ref() {
                        let self_fmri = package.fmri_ref().clone();
                        match fmri.cmp(package.fmri_ref()) {
                            Ordering::Less => {
                                return Some(self_fmri);
                            }
                            Ordering::Greater | Ordering::Equal => {}
                        }
                    }
                }
            }
        }
        None
    }

    pub fn check_require_dependency(&self, fmri: &FMRI, checking_fmri: &FMRI) -> bool {
        if fmri.package_name_eq(checking_fmri) {
            match fmri.cmp(checking_fmri) {
                Ordering::Equal | Ordering::Less => {
                    return self.is_there_newer_version(checking_fmri).is_none();
                }
                // dependency need greater version of fmri
                Ordering::Greater => {}
            };
        }
        false
    }

    pub fn check_if_renamed_needs_renamed(&self, problems: &mut Problems) {
        let mut find_needed_package_closure =
            |dependency: &Dependency, package_versions: &PackageVersions| match dependency
                .get_ref()
                .get_content_ref()
            {
                Ok(fmri) => {
                    if let Some(needed_package_versions) = self.get_package_versions_from_fmri(fmri)
                    {
                        if needed_package_versions.is_renamed() {
                            problems.add_problem(RenamedNeedsRenamed(
                                package_versions.fmri_ref().clone(),
                                needed_package_versions.fmri(),
                            ));
                        }
                    }
                }
                Err(fmri_list) => {
                    for fmri in fmri_list.get_ref() {
                        match self.get_package_versions_from_fmri(fmri) {
                            None => {}
                            Some(needed_package_versions) => {
                                if needed_package_versions.is_renamed() {
                                    problems.add_problem(RenamedNeedsRenamed(
                                        package_versions.fmri_ref().clone(),
                                        needed_package_versions.fmri(),
                                    ));
                                }
                            }
                        }
                    }
                }
            };

        for component in self.get_ref() {
            for package_versions in component.get_versions_ref() {
                if package_versions.is_renamed() {
                    let package = package_versions.get_packages_ref().last().unwrap();

                    for runtime in package.get_runtime_dependencies() {
                        find_needed_package_closure(runtime, package_versions)
                    }

                    for build in package.get_build_dependencies() {
                        find_needed_package_closure(build, package_versions)
                    }

                    for test in package.get_test_dependencies() {
                        find_needed_package_closure(test, package_versions)
                    }

                    for system_build in package.get_system_build_dependencies() {
                        find_needed_package_closure(system_build, package_versions)
                    }

                    for system_test in package.get_system_test_dependencies() {
                        find_needed_package_closure(system_test, package_versions)
                    }
                }
            }
        }
    }

    pub fn get_package_versions_from_fmri(&self, fmri: &FMRI) -> Option<PackageVersions> {
        for component in self.get_ref() {
            for package_versions in component.get_versions_ref() {
                if package_versions.fmri_ref().package_name_eq(fmri) {
                    return Some(package_versions.clone());
                }
            }
        }

        None
    }

    pub fn check_if_fmri_exists_as_package(&self, fmri: &FMRI) -> bool {
        for component in self.get_ref() {
            for package_versions in component.get_versions_ref() {
                if package_versions.fmri_ref().package_name_eq(fmri) {
                    return true;
                }
            }
        }
        false
    }

    pub fn add(&mut self, component: Component) {
        self.components.push(component)
    }

    pub fn get_obsoleted_ref(&self) -> &FMRIList {
        &self.obsolete
    }

    pub fn add_obsoleted(&mut self, fmri: FMRI) {
        self.obsolete.add(fmri)
    }

    pub fn is_fmri_obsoleted(&self, fmri: &FMRI) -> bool {
        self.obsolete.contains(fmri)
    }

    pub fn change(&mut self, new_components: Vec<Component>) {
        self.components = new_components
    }

    pub fn get(self) -> Vec<Component> {
        self.components
    }

    pub fn get_ref(&self) -> &Vec<Component> {
        &self.components
    }

    pub fn get_ref_mut(&mut self) -> &mut Vec<Component> {
        &mut self.components
    }
}

impl Default for Components {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Components {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string: String = "".to_owned();

        for (index, component) in self.get_ref().iter().enumerate() {
            string.push_str(&format!(
                "{}. component: {}\n",
                index + 1,
                component.get_name_ref()
            ));
            for package_versions in component.get_versions_ref() {
                string.push_str(&format!(
                    "  package_name: {}\n",
                    package_versions.fmri_ref()
                ));
                for package in package_versions.get_packages_ref() {
                    string.push_str(&format!("    package: {}\n", package.fmri_ref()));
                    for i in 0..5 {
                        match i {
                            0 => {
                                let dp = package.get_runtime_dependencies().iter().enumerate();
                                if dp.len() != 0 {
                                    string.push_str("      runtime dependencies:\n");
                                    for (index, dependency) in dp {
                                        string.push_str(&format!(
                                            "        {}. {}\n",
                                            index + 1,
                                            dependency.get_ref()
                                        ));
                                    }
                                }
                            }
                            1 => {
                                let dp = package.get_build_dependencies().iter().enumerate();
                                if dp.len() != 0 {
                                    string.push_str("      build dependencies:\n");
                                    for (index, dependency) in dp {
                                        string.push_str(&format!(
                                            "        {}. {}\n",
                                            index + 1,
                                            dependency.get_ref()
                                        ));
                                    }
                                }
                            }
                            2 => {
                                let dp = package.get_test_dependencies().iter().enumerate();
                                if dp.len() != 0 {
                                    string.push_str("      test dependencies:\n");
                                    for (index, dependency) in dp {
                                        string.push_str(&format!(
                                            "        {}. {}\n",
                                            index + 1,
                                            dependency.get_ref()
                                        ));
                                    }
                                }
                            }
                            3 => {
                                let dp = package.get_system_build_dependencies().iter().enumerate();
                                if dp.len() != 0 {
                                    string.push_str("      system build dependencies:\n");
                                    for (index, dependency) in dp {
                                        string.push_str(&format!(
                                            "        {}. {}\n",
                                            index + 1,
                                            dependency.get_ref()
                                        ));
                                    }
                                }
                            }
                            4 => {
                                let dp = package.get_system_test_dependencies().iter().enumerate();
                                if dp.len() != 0 {
                                    string.push_str("      system test dependencies:\n");
                                    for (index, dependency) in dp {
                                        string.push_str(&format!(
                                            "        {}. {}\n",
                                            index + 1,
                                            dependency.get_ref()
                                        ));
                                    }
                                }
                            }
                            _ => panic!(),
                        }
                    }
                }
            }
        }

        write!(f, "{}", string)
    }
}
