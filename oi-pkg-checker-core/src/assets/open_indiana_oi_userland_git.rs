use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use fmri::{fmri_list::FMRIList, FMRI};

use crate::problems::Problem::NonExistingPackageInPkg5;
use crate::{
    assets::catalogs_c::open_json_file,
    problems::{
        Problem::{
            MissingComponentForPackage, ObsoletedPackageInComponent, PackageInMultipleComponents,
            RenamedPackageInComponent, UnRunnableMakeCommand,
        },
        Problems,
    },
    Components, Dependencies, DependencyTypes,
    DependencyTypes::{Build, SystemBuild, SystemTest, Test},
    PackageVersions,
};

#[derive(Clone, Debug)]
pub struct ComponentPackagesList(Vec<ComponentPackages>);

#[derive(Clone, Debug)]
pub struct ComponentPackages {
    pub component_name: String,
    pub path_to_component: PathBuf,
    pub packages_in_component: FMRIList,
}

impl ComponentPackagesList {
    pub fn new(oi_userland_components: &Path) -> Self {
        let components_path = oi_userland_components.to_string_lossy();

        let _output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && rm -f components.mk ; gmake COMPONENTS_IGNORE=/dev/null components.mk",
                components_path
            ))
            .output()
            .expect("failed to run command");

        // TODO: check output validity
        // println!("{:?}", a);

        let output = Command::new("cat")
            .arg(format!("{}/components.mk", components_path.clone()))
            .output()
            .expect("failed to run command");

        let mut component_packages_list: Self = Self(vec![]);

        for line in String::from_utf8(output.stdout).unwrap().split('\n') {
            if line.is_empty() {
                continue;
            }

            let component_name = line.split_whitespace().last().unwrap().to_owned();

            let path_to_component =
                PathBuf::from(format!("{}/{}", components_path, component_name));

            let mut packages_in_component = FMRIList::new();
            for fmri in open_json_file(
                PathBuf::from(format!(
                    "{}/pkg5",
                    path_to_component.clone().to_string_lossy()
                )), // pkg5 location
            )
            .as_object()
            .expect("expect object")
            .get("fmris")
            .expect("expect fmris")
            .as_array()
            .expect("expect array")
            {
                packages_in_component
                    .add(FMRI::parse_raw(fmri.as_str().expect("expect string")).unwrap())
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

    pub fn get_component_packages_of_package_versions(
        &self,
        problems: &mut Problems,
        package_versions: &PackageVersions,
    ) -> Option<ComponentPackages> {
        for component_packages in &self.0 {
            for fmri in component_packages.packages_in_component.get_ref() {
                if fmri.package_name_eq(package_versions.fmri_ref()) {
                    if package_versions.is_renamed() || package_versions.is_obsolete() {
                        if package_versions.is_renamed() {
                            problems.add_problem(RenamedPackageInComponent(
                                package_versions.clone().fmri(),
                                component_packages.component_name.clone(),
                            ));
                        } else {
                            problems.add_problem(ObsoletedPackageInComponent(
                                package_versions.clone().fmri(),
                                component_packages.component_name.clone(),
                            ));
                        }

                        return None;
                    }

                    return Some(component_packages.clone());
                }
            }
        }

        if !package_versions.is_obsolete() && !package_versions.is_renamed() {
            problems.add_problem(MissingComponentForPackage(package_versions.clone().fmri()));
        }

        None
    }

    fn get_dependencies_of_component(
        &self,
        problems: &mut Problems,
        component_path: PathBuf,
        dependencies_type: &DependencyTypes,
    ) -> Result<FMRIList, ()> {
        let mut make_command: String = "gmake ".to_owned();

        #[cfg(target_os = "linux")]
        make_command.push_str("GSED=/usr/bin/sed ");

        make_command.push_str(match dependencies_type {
            Build => "print-value-REQUIRED_PACKAGES",
            Test => "print-value-TEST_REQUIRED_PACKAGES",
            SystemBuild => "print-value-USERLAND_REQUIRED_PACKAGES",
            SystemTest => "print-value-USERLAND_TEST_REQUIRED_PACKAGES",
            _ => panic!(),
        });

        let command = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && {}",
                component_path.to_string_lossy(),
                make_command
            ))
            .output()
            .expect("failed to run command");

        if command.status.code().unwrap() != 0 {
            problems.add_problem(UnRunnableMakeCommand(
                make_command.to_owned(),
                component_path,
            ));

            return Err(());
        }

        let binding = String::from_utf8(command.stdout).unwrap();

        let fmri_list: Vec<FMRI> = binding
            .split_whitespace()
            .map(|fmri| FMRI::parse_raw(fmri).unwrap())
            .collect();

        Ok(FMRIList::from(fmri_list))
    }

    /// finds same package in multiple components
    pub fn same_packages_in_components(&self, problems: &mut Problems) {
        let mut map: HashMap<&FMRI, Vec<&String>> = HashMap::new();

        for component_packages in self.get() {
            for fmri in component_packages.packages_in_component.get_ref() {
                map.entry(fmri)
                    .or_default()
                    .push(&component_packages.component_name)
            }
        }

        for (fmri, components) in map {
            if components.len() > 1 {
                problems.add_problem(PackageInMultipleComponents(
                    fmri.clone(),
                    components.into_iter().cloned().collect::<Vec<String>>(),
                ));
            }
        }
    }

    pub fn non_existing_packages_in_pkg5(&self, problems: &mut Problems, components: &Components) {
        for a in self.get() {
            for fmri in a.packages_in_component.get_ref() {
                if components.is_fmri_obsoleted(fmri) {
                    continue;
                }

                if components.check_if_fmri_exists_as_package(fmri) {
                    continue;
                }

                problems.add_problem(NonExistingPackageInPkg5(
                    fmri.clone(),
                    a.path_to_component.clone(),
                ))
            }
        }
    }
}

pub fn load_dependencies(
    components: &mut Components,
    problems: &mut Problems,
    component_packages_list: &ComponentPackagesList,
    dependencies_type: &DependencyTypes,
) {
    for component in components.get_ref_mut() {
        for packet_versions in component.get_versions_ref_mut() {
            if let Some(component_path) = component_packages_list
                .get_component_packages_of_package_versions(problems, packet_versions)
                .map(|component_packages| component_packages.path_to_component)
            {
                if let Ok(fmri_list) = component_packages_list.get_dependencies_of_component(
                    problems,
                    component_path,
                    dependencies_type,
                ) {
                    let deps = Dependencies::new_from_fmri_list(fmri_list);

                    for package in packet_versions.get_packages_ref_mut() {
                        match dependencies_type {
                            Build => package.add_build_dependencies(deps.clone()),
                            Test => package.add_test_dependencies(deps.clone()),
                            SystemBuild => package.add_system_build_dependencies(deps.clone()),
                            SystemTest => package.add_system_test_dependencies(deps.clone()),
                            _ => panic!("unsupported dependency type"),
                        }
                    }
                }
            }
        }
    }
}

pub fn component_list(
    components: &mut Components,
    problems: &mut Problems,
    component_packages_list: &ComponentPackagesList,
) {
    let mut new_components = Components::new();

    for component in components.get_ref() {
        for package_version in component.get_versions_ref() {
            new_components.add_package_to_component_with_name(
                package_version,
                component_packages_list
                    .get_component_packages_of_package_versions(problems, package_version)
                    .map(|component_packages| component_packages.component_name)
                    .unwrap_or_else(|| "".to_owned()),
            )
        }
    }

    new_components.name_unnamed_components();
    components.change(new_components.get());
}
