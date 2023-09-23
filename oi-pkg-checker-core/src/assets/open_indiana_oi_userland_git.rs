use std::path::PathBuf;
use fmri::fmri_list::FMRIList;
use crate::packages::components::Components;
use std::process::Command;
use crate::assets::catalog_encumbered_dependency_c::open_json_file;
use crate::assets::open_indiana_oi_userland_git::Errors::{First, Second};
use fmri::FMRI;
use crate::packages::dependencies::Dependencies;
use crate::packages::dependency_type::DependencyTypes;
use crate::packages::package_versions::PackageVersions;
use crate::problems::{MissingComponentForPackage, MissingComponentForPackageList, RenamedPackageInComponent, ProblemList, UnRunnableMakeCommand, UnRunnableMakeCommandList, RenamedPackageInComponentList};

#[derive(Clone)]
pub struct ComponentPackagesList(Vec<ComponentPackages>);

#[derive(Clone)]
pub struct ComponentPackages {
    component_name: String,
    path_to_component: PathBuf,
    packages_in_component: FMRIList
}

impl ComponentPackages {
    pub fn get_component_name(&self) -> &String {
        &self.component_name
    }

    pub fn get_packages_in_component(&self) -> &FMRIList {
        &self.packages_in_component
    }
}

impl ComponentPackagesList {
    pub fn new(oi_userland_components: &PathBuf, oi_userland_components_encumbered: &PathBuf) -> Self {
        let components_path = oi_userland_components.to_string_lossy();

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && rm -f components.mk ; make components.mk", components_path))
            .output()
            .expect("failed to run command");
        let e_o = vec![71, 101, 110, 101, 114, 97, 116, 105, 110, 103, 32, 99, 111, 109, 112, 111, 110, 101, 110, 116, 32, 108, 105, 115, 116, 46, 46, 46, 10, 109, 97, 107, 101, 58, 32, 78, 111, 116, 104, 105, 110, 103, 32, 116, 111, 32, 98, 101, 32, 100, 111, 110, 101, 32, 102, 111, 114, 32, 39, 99, 111, 109, 112, 111, 110, 101, 110, 116, 115, 46, 109, 107, 39, 46, 10];
        if output.stdout != e_o {
            panic!("output of cd oi_userland_components && rm -f components.mk ; make components.mk is not\n{}\nbut:\n{}", String::from_utf8(e_o).unwrap(), String::from_utf8(output.stdout).unwrap())
        }

        let output = Command::new("cat")
            .arg(format!("{}/components.mk", components_path.clone()))
            .output()
            .expect("failed to run command");

        let mut component_packages_list: Self = Self(vec![]);

        for line in String::from_utf8(output.stdout).unwrap().split("\n") {
            if line == "" {
                // TODO: ???
                let component_name = "openindiana/illumos-gate".to_owned();
                let path_to_component = PathBuf::from(format!("{}/{}", components_path, component_name));

                let mut packages_in_component = FMRIList::new();
                for fmri in open_json_file(
                    PathBuf::from(format!("{}/pkg5", path_to_component.clone().to_string_lossy())) // pkg5 location
                )
                    .as_object()
                    .expect("expect object")
                    .get("fmris")
                    .expect("expect fmris")
                    .as_array()
                    .expect("expect array") {

                    packages_in_component.add(FMRI::parse_raw(&fmri.as_str().expect("expect string").to_owned()))
                }

                component_packages_list.0.push(ComponentPackages {
                    component_name,
                    path_to_component,
                    packages_in_component,
                });

                break
            }

            let component_name = line.split_whitespace().last().unwrap().to_owned();
            let path_to_component = PathBuf::from(format!("{}/{}", components_path, component_name));

            let mut packages_in_component = FMRIList::new();
            for fmri in open_json_file(
                PathBuf::from(format!("{}/pkg5", path_to_component.clone().to_string_lossy())) // pkg5 location
            )
                .as_object()
                .expect("expect object")
                .get("fmris")
                .expect("expect fmris")
                .as_array()
                .expect("expect array") {

                packages_in_component.add(FMRI::parse_raw(&fmri.as_str().expect("expect string").to_owned()))
            }

            component_packages_list.0.push(ComponentPackages {
                component_name,
                path_to_component,
                packages_in_component,
            });
        }

        let components_path_encumbered = oi_userland_components_encumbered.to_string_lossy();
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && rm -f components.mk ; make components.mk", components_path_encumbered))
            .output()
            .expect("failed to run command");
        let e_o = vec![71, 101, 110, 101, 114, 97, 116, 105, 110, 103, 32, 99, 111, 109, 112, 111, 110, 101, 110, 116, 32, 108, 105, 115, 116, 46, 46, 46, 10, 109, 97, 107, 101, 58, 32, 78, 111, 116, 104, 105, 110, 103, 32, 116, 111, 32, 98, 101, 32, 100, 111, 110, 101, 32, 102, 111, 114, 32, 39, 99, 111, 109, 112, 111, 110, 101, 110, 116, 115, 46, 109, 107, 39, 46, 10];
        if output.stdout != e_o {
            panic!("output of cd oi_userland_components && rm -f components.mk ; make components.mk is not\n{}\nbut:\n{}", String::from_utf8(e_o).unwrap(), String::from_utf8(output.stdout).unwrap())
        }

        let output = Command::new("cat")
            .arg(format!("{}/components.mk", components_path_encumbered.clone()))
            .output()
            .expect("failed to run command");

        for line in String::from_utf8(output.stdout).unwrap().split("\n") {
            if line == "" {
                break
            }

            let component_name = line.split_whitespace().last().unwrap().to_owned();
            let path_to_component = PathBuf::from(format!("{}/{}", components_path_encumbered, component_name));

            let mut packages_in_component = FMRIList::new();
            for fmri in open_json_file(
                PathBuf::from(format!("{}/pkg5", path_to_component.clone().to_string_lossy())) // pkg5 location
            )
                .as_object()
                .expect("expect object")
                .get("fmris")
                .expect("expect fmris")
                .as_array()
                .expect("expect array") {

                packages_in_component.add(FMRI::parse_raw(&fmri.as_str().expect("expect string").to_owned()))
            }

            component_packages_list.0.push(ComponentPackages {
                component_name,
                path_to_component,
                packages_in_component,
            });
        }

        component_packages_list
    }

    pub fn get(&self) -> &Vec<ComponentPackages> {
        &self.0
    }

    pub fn get_component_name_of_package_fmri(&self, comparing_to: &FMRI) -> Option<String> {
        for component_packages in self.get() {
            for fmri in component_packages.packages_in_component.get_ref() {

                if comparing_to.get_package_name_as_ref_string() == "library/python/setuptools-37" {
                    println!("{}", fmri);
                }

                if fmri.package_name_eq(comparing_to) {
                    return Some(component_packages.component_name.clone())
                }
            }
        }

        None
    }

    fn get_component_path_of_package_versions(&self, package_versions: &PackageVersions) -> Result<PathBuf, Option<Errors>> {
        for component_packages in self.get() {
            for fmri in component_packages.packages_in_component.get_ref() {
                if fmri.package_name_eq(package_versions.fmri_ref()) {
                    if package_versions.is_renamed() {
                        return Err(Some(First(RenamedPackageInComponent::new(package_versions.clone().fmri(), true, component_packages.component_name.clone()))));
                    } else if package_versions.is_obsolete() {
                        return Err(Some(First(RenamedPackageInComponent::new(package_versions.clone().fmri(), false, component_packages.component_name.clone()))));
                    }

                    return Ok(component_packages.path_to_component.clone())
                }
            }
        }

        if !package_versions.is_obsolete() && !package_versions.is_renamed() {
            return Err(Some(Second(MissingComponentForPackage::new(package_versions.clone().fmri()))));
        }
        Err(None)
    }

    pub fn get_component_name_of_package_versions(&self, package_versions: &PackageVersions) -> Result<String, Option<Errors>> {
        for component_packages in self.get() {
            for fmri in component_packages.packages_in_component.get_ref() {
                if fmri.package_name_eq(package_versions.fmri_ref()) {
                    if package_versions.is_renamed() {
                        return Err(Some(First(RenamedPackageInComponent::new(package_versions.clone().fmri(), true, component_packages.component_name.clone()))));
                    } else if package_versions.is_obsolete() {
                        return Err(Some(First(RenamedPackageInComponent::new(package_versions.clone().fmri(), false, component_packages.component_name.clone()))));
                    }

                    return Ok(component_packages.component_name.clone())
                }
            }
        }

        if !package_versions.is_obsolete() && !package_versions.is_renamed() {
            return Err(Some(Second(MissingComponentForPackage::new(package_versions.clone().fmri()))));
        }
        Err(None)
    }

    fn get_dependencies_of_component(&self, component_path: PathBuf, dependencies_type: &DependencyTypes) -> Result<FMRIList, UnRunnableMakeCommand> {
        let make_command = match dependencies_type {
            DependencyTypes::Build => "make print-value-REQUIRED_PACKAGES",
            DependencyTypes::Test => "make print-value-TEST_REQUIRED_PACKAGES",
            DependencyTypes::SystemBuild => "make print-value-USERLAND_REQUIRED_PACKAGES",
            DependencyTypes::SystemTest => "make print-value-USERLAND_TEST_REQUIRED_PACKAGES",
            _ => panic!()
        };

        let command = Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && {}", component_path.to_string_lossy(), make_command))
            .output()
            .expect("failed to run command");

        if command.status.code().unwrap() != 0 {
            // error!("bad command exit status: {} for command: {} in {}", command.status, make_command, component_path.to_string_lossy());
            return Err(UnRunnableMakeCommand::new(make_command.to_owned(), component_path))
        }

        let binding = String::from_utf8(command.stdout).unwrap();

        let mut fmri_list = FMRIList::new();
        for fmri in binding.trim().split_whitespace() {
            fmri_list.add(FMRI::parse_raw(&fmri.to_owned()))
        }
        Ok(fmri_list)
    }
}

// TODO: huh what is this, remove it
pub enum Errors {
    First(RenamedPackageInComponent),
    Second(MissingComponentForPackage)
}

pub fn build_dependencies(components: &mut Components, component_packages_list: ComponentPackagesList) -> Result<(), (MissingComponentForPackageList, RenamedPackageInComponentList, UnRunnableMakeCommandList)> {
    load_dependencies(components, component_packages_list, &DependencyTypes::Build)
}

pub fn test_dependencies(components: &mut Components, component_packages_list: ComponentPackagesList) -> Result<(), (MissingComponentForPackageList, RenamedPackageInComponentList, UnRunnableMakeCommandList)> {
    load_dependencies(components, component_packages_list, &DependencyTypes::Test)
}

pub fn system_build_dependencies(components: &mut Components, component_packages_list: ComponentPackagesList) -> Result<(), (MissingComponentForPackageList, RenamedPackageInComponentList, UnRunnableMakeCommandList)> {
    load_dependencies(components, component_packages_list, &DependencyTypes::SystemBuild)
}

pub fn system_test_dependencies(components: &mut Components, component_packages_list: ComponentPackagesList) -> Result<(), (MissingComponentForPackageList, RenamedPackageInComponentList, UnRunnableMakeCommandList)> {
    load_dependencies(components, component_packages_list, &DependencyTypes::SystemTest)
}

fn load_dependencies(
    components: &mut Components,
    component_packages_list: ComponentPackagesList,
    dependencies_type: &DependencyTypes
) -> Result<(), (MissingComponentForPackageList, RenamedPackageInComponentList, UnRunnableMakeCommandList)> {

    let mut missing_component_for_package_list = MissingComponentForPackageList::new();
    let mut renamed_package_in_component_list = RenamedPackageInComponentList::new();
    let mut un_runnable_make_command_list = UnRunnableMakeCommandList::new();

    for component in components.get_ref_mut() {
        for packet_versions in component.get_versions_ref_mut() {
            match component_packages_list.get_component_path_of_package_versions(packet_versions) {
                Ok(component_path) => {
                    let fmri_list = match component_packages_list.get_dependencies_of_component(
                        component_path,
                        dependencies_type
                    ) {
                        Ok(fmri_list) => fmri_list,
                        Err(error) => {
                            un_runnable_make_command_list.add(error);
                            continue;
                        }
                    };
                    
                    for package in packet_versions.get_packages_ref_mut() {
                        match dependencies_type {
                            DependencyTypes::Build => {
                                package.add_build_dependencies(Dependencies::new_from_fmri_list(fmri_list.clone()))
                            },
                            DependencyTypes::Test => package.add_test_dependencies(Dependencies::new_from_fmri_list(fmri_list.clone())),
                            DependencyTypes::SystemBuild => package.add_system_build_dependencies(Dependencies::new_from_fmri_list(fmri_list.clone())),
                            DependencyTypes::SystemTest => package.add_system_test_dependencies(Dependencies::new_from_fmri_list(fmri_list.clone())),
                            _ => panic!()
                        }
                    }
                }
                Err(error) => {
                    match error {
                        None => {}
                        Some(error) => {
                            match error {
                                First(renamed_package_in_component) => renamed_package_in_component_list.add(renamed_package_in_component),
                                Second(missing_component_for_package) => missing_component_for_package_list.add(missing_component_for_package)
                            }
                        }
                    }
                }
            }
        }
    }

    if !missing_component_for_package_list.is_empty() || !un_runnable_make_command_list.is_empty() || !renamed_package_in_component_list.is_empty() {
        return Err((missing_component_for_package_list, renamed_package_in_component_list, un_runnable_make_command_list));
    }

    Ok(())
}

pub fn component_list(
    components: &mut Components,
    component_packages_list: ComponentPackagesList
) -> Result<(), (MissingComponentForPackageList, RenamedPackageInComponentList)> {
    let mut new_components = Components::new();
    let mut missing_component_for_package_list = MissingComponentForPackageList::new();
    let mut renamed_package_in_component_list = RenamedPackageInComponentList::new();

    for component in components.get_ref() {
        for package_version in component.get_versions_ref() {
            let component_name = match component_packages_list.get_component_name_of_package_versions(package_version) {
                Ok(name) => name,
                Err(error) => {
                    match error {
                        None => {}
                        Some(error) => match error {
                            First(renamed_package_in_component) => renamed_package_in_component_list.add(renamed_package_in_component),
                            Second(missing_component_for_package) => missing_component_for_package_list.add(missing_component_for_package)
                        }
                    }
                    "".to_owned()
                }
            };
            new_components.add_package_to_component_with_name(package_version, component_name)
        }
    }

    new_components.name_unnamed_components();

    renamed_package_in_component_list += check_for_renamed_package_in_component_list(components, component_packages_list);

    components.change(new_components.get());
    if !missing_component_for_package_list.is_empty() || !renamed_package_in_component_list.is_empty() {
        return Err((missing_component_for_package_list, renamed_package_in_component_list));
    }
    Ok(())
}

fn check_for_renamed_package_in_component_list(components: &Components, component_packages_list: ComponentPackagesList) -> RenamedPackageInComponentList {
    let mut renamed_package_in_component_list = RenamedPackageInComponentList::new();
    for obsoleted in components.get_obsoleted_ref().get_ref() {
        match component_packages_list.get_component_name_of_package_versions(&PackageVersions::new(obsoleted.clone())) {
            Ok(_) => {}
            Err(error) => match error {
                None => {}
                Some(errors) => match errors {
                    First(renamed_package_in_component) => {
                        println!("added");
                        renamed_package_in_component_list.add(renamed_package_in_component);
                    },
                    Second(_) => {}
                }
            }
        }
    }

    renamed_package_in_component_list
}