use std::path::PathBuf;
use fmri::fmri_list::FMRIList;
use crate::packages::components::Components;
use std::process::Command;
use crate::assets::catalogs_c::open_json_file;
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
    pub fn new(oi_userland_components: &PathBuf) -> Self {
        let components_path = oi_userland_components.to_string_lossy();

        Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && rm -f components.mk ; gmake COMPONENTS_IGNORE=/dev/null components.mk", components_path))
            .output()
            .expect("failed to run command");

        let output = Command::new("cat")
            .arg(format!("{}/components.mk", components_path.clone()))
            .output()
            .expect("failed to run command");

        let mut component_packages_list: Self = Self(vec![]);

        for line in String::from_utf8(output.stdout).unwrap().split("\n") {
            if line == "" { continue; }

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

        component_packages_list
    }

    pub fn get(&self) -> &Vec<ComponentPackages> {
        &self.0
    }

    pub fn get_component_name_of_package_fmri(&self, comparing_to: &FMRI) -> Option<String> {
        for component_packages in self.get() {
            for fmri in component_packages.packages_in_component.get_ref() {
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
        let mut make_command: String = "gmake ".to_owned();

        #[cfg(target_os = "linux")]
        make_command.push_str("GSED=/usr/bin/sed ");

        make_command.push_str(match dependencies_type {
            DependencyTypes::Build => "print-value-REQUIRED_PACKAGES",
            DependencyTypes::Test => "print-value-TEST_REQUIRED_PACKAGES",
            DependencyTypes::SystemBuild => "print-value-USERLAND_REQUIRED_PACKAGES",
            DependencyTypes::SystemTest => "print-value-USERLAND_TEST_REQUIRED_PACKAGES",
            _ => panic!()
        });

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