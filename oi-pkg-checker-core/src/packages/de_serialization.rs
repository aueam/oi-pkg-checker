use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use bincode::{deserialize, serialize};
use fmri::FMRI;
use serde::{Deserialize, Serialize};

use crate::{
    clone, downgrade, get, get_mut, new,
    packages::{
        package::{Package as OrgPackage, PackageVersion},
        rev_depend_type::RevDependType,
    },
    shared_type, weak_type, Component as OrgComponent, Components as OrgComponents, Problems,
};

#[derive(Serialize, Deserialize)]
pub struct Components {
    components: Vec<Component>,
    packages: Vec<Package>,
    problems: Problems,
}
#[derive(Serialize, Deserialize)]
pub struct Component {
    name: String,
    packages: Vec<FMRI>,
    build: Vec<FMRI>,
    test: Vec<FMRI>,
    sys_build: Vec<FMRI>,
    sys_test: Vec<FMRI>,
}
#[derive(Serialize, Deserialize)]
pub struct Package {
    fmri: FMRI,
    versions: Vec<PackageVersion>,
    component: Option<String>,
    obsolete: bool,
    renamed: bool,
    runtime_dependents: Vec<RevDependType>,
    build_dependents: Vec<String>,
    test_dependents: Vec<String>,
    sys_build_dependents: Vec<String>,
    sys_test_dependents: Vec<String>,
}

impl OrgComponents {
    pub fn deserialize<P: AsRef<Path> + ?Sized + Display>(path: &P) -> Result<Self, String> {
        let data = &mut Vec::new();
        File::open(path)
            .map_err(|e| format!("failed to open file {}: {}", path, e))?
            .read_to_end(data)
            .map_err(|e| format!("failed to read file: {}", e))?;
        let components: Components =
            deserialize(data).map_err(|e| format!("failed to deserialize data: {}", e))?;

        let mut org_components = OrgComponents {
            problems: components.problems,
            ..Default::default()
        };

        for package in &components.packages {
            let p = new!(OrgPackage {
                fmri: package.fmri.clone(),
                versions: package.versions.clone(),
                component: None,
                obsolete: package.obsolete,
                renamed: package.renamed,
                runtime_dependents: package.runtime_dependents.clone(),
                build_dependents: Vec::new(),
                test_dependents: Vec::new(),
                sys_build_dependents: Vec::new(),
                sys_test_dependents: Vec::new(),
            });

            org_components.packages.push(clone!(&p));
            org_components.hash_packages.insert(
                package.fmri.clone().get_package_name_as_string(),
                clone!(&p),
            );
        }

        for component in components.components {
            let c = |v: Vec<FMRI>| -> Vec<weak_type!(OrgPackage)> {
                v.iter()
                    .map(|f| downgrade!(org_components.get_package_by_fmri(f).unwrap()))
                    .collect()
            };
            let a = new!(OrgComponent {
                name: component.name.clone(),
                packages: c(component.packages),
                build: c(component.build),
                test: c(component.test),
                sys_build: c(component.sys_build),
                sys_test: c(component.sys_test),
            });

            org_components.components.push(clone!(&a));
            org_components
                .hash_components
                .insert(component.name, clone!(&a));
        }

        for p in &components.packages {
            let mut package = get_mut!(org_components.get_package_by_fmri(&p.fmri).unwrap());

            package.component = p
                .component
                .as_ref()
                .map(|name| clone!(org_components.get_component_by_name(name).unwrap()));

            let c = |cs: Vec<String>| -> Vec<shared_type!(OrgComponent)> {
                cs.iter()
                    .map(|name| clone!(org_components.get_component_by_name(name).unwrap()))
                    .collect()
            };

            package.build_dependents = c(p.build_dependents.clone());
            package.test_dependents = c(p.test_dependents.clone());
            package.sys_build_dependents = c(p.sys_build_dependents.clone());
            package.sys_test_dependents = c(p.sys_test_dependents.clone());
        }

        Ok(org_components)
    }

    pub fn serialize<P: AsRef<Path> + ?Sized + Display>(&self, path: &P) -> Result<(), String> {
        let mut components = Components {
            packages: Vec::new(),
            components: Vec::new(),
            problems: self.problems.clone(),
        };

        let cn = |c: shared_type!(OrgComponent)| -> String { get!(c).get_name().clone() };
        let cnr = |c: &shared_type!(OrgComponent)| -> String { get!(c).get_name().clone() };
        let f = |p: &weak_type!(OrgPackage)| -> FMRI { get!(p.upgrade().unwrap()).fmri.clone() };

        for p in &self.packages {
            let package = get!(p);
            components.packages.push(Package {
                fmri: package.fmri.clone(),
                versions: package.versions.clone(),
                component: package.component.clone().map(cn),
                obsolete: package.is_obsolete(),
                renamed: package.is_renamed(),
                runtime_dependents: package.runtime_dependents.clone(),
                build_dependents: package.build_dependents.iter().map(cnr).collect(),
                test_dependents: package.test_dependents.iter().map(cnr).collect(),
                sys_build_dependents: package.sys_build_dependents.iter().map(cnr).collect(),
                sys_test_dependents: package.sys_test_dependents.iter().map(cnr).collect(),
            });
        }

        for c in &self.components {
            let component = get!(c);
            components.components.push(Component {
                name: component.get_name().clone(),
                packages: component.packages.iter().map(f).collect(),
                build: component.build.iter().map(f).collect(),
                test: component.test.iter().map(f).collect(),
                sys_build: component.sys_build.iter().map(f).collect(),
                sys_test: component.sys_test.iter().map(f).collect(),
            });
        }

        File::create(path)
            .map_err(|e| format!("failed to create file {}: {}", path, e))?
            .write_all(
                &serialize(&components)
                    .map_err(|e| format!("failed to serialize components: {}", e))?,
            )
            .map_err(|e| format!("failed to write serialized data into file: {}", e))?;
        Ok(())
    }
}
