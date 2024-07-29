use std::{cell::RefCell, fs, rc::Rc};

use fmri::{FMRI, Version};

use crate::{
    Component,
    Components, DependTypes, packages::{
        dependency_type::DependencyTypes,
        package::{Package, PackageVersion},
    },
};

const PATH: &str = "/tmp/rust-oi-pkg-checker-core-de-serialization-test.bin";

#[test]
fn serialization() {
    new_data().serialize(PATH).unwrap();
    let c1 = Components::deserialize(PATH).unwrap();
    let c2 = new_valid_data();

    assert_eq!(
        format!("{:#?}", c1.components),
        format!("{:#?}", c2.components)
    );
    assert_eq!(format!("{:#?}", c1.packages), format!("{:#?}", c2.packages));
    assert_eq!(format!("{:#?}", c1.problems), format!("{:#?}", c2.problems));

    for (name, c1) in &c1.hash_components {
        let c2 = c2.get_component_by_name(name).unwrap();
        assert_eq!(format!("{:#?}", c1), format!("{:#?}", c2));
    }

    for (name, p1) in &c1.hash_packages {
        let p2 = c2
            .get_package_by_fmri(&FMRI::parse_raw(name).unwrap())
            .unwrap();
        assert_eq!(format!("{:#?}", p1), format!("{:#?}", p2));
    }

    fs::remove_file(PATH).unwrap();
}

fn new_valid_data() -> Components {
    let p1_fmri = FMRI::parse_raw("p1").unwrap();
    let p2_fmri = FMRI::parse_raw("p2").unwrap();
    let p3_fmri = FMRI::parse_raw("p3").unwrap();
    let p4_fmri = FMRI::parse_raw("p4").unwrap();
    let p = |f: FMRI,
             versions: Vec<PackageVersion>,
             obsolete: bool,
             renamed: bool|
     -> Rc<RefCell<Package>> {
        Rc::new(RefCell::new(Package {
            fmri: f,
            versions,
            component: None,
            obsolete,
            renamed,
            runtime_dependents: vec![],
            build_dependents: vec![],
            test_dependents: vec![],
            sys_build_dependents: vec![],
            sys_test_dependents: vec![],
        }))
    };
    let mut p1_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p1_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    let mut p2_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p2_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    let mut p3_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p3_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    let mut p4_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p4_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    p1_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p3_fmri.clone())]);
    p1_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p3_fmri.clone())]);
    p2_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p4_fmri.clone())]);
    p2_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p4_fmri.clone())]);
    p3_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p1_fmri.clone())]);
    p3_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p1_fmri.clone())]);
    p4_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p2_fmri.clone())]);
    p4_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p2_fmri.clone())]);
    p1_v2.set_obsolete(true);
    p1_v1.set_renamed(true);
    p2_v1.set_obsolete(true);
    p2_v2.set_renamed(true);
    let p1 = p(p1_fmri.clone(), vec![p1_v1, p1_v2], true, false);
    let p2 = p(p2_fmri.clone(), vec![p2_v1, p2_v2], false, true);
    let p3 = p(p3_fmri.clone(), vec![p3_v1, p3_v2], false, false);
    let p4 = p(p4_fmri.clone(), vec![p4_v1, p4_v2], false, false);
    let mut components = Components::default();
    components.packages.push(Rc::clone(&p1));
    components.packages.push(Rc::clone(&p2));
    components.packages.push(Rc::clone(&p3));
    components.packages.push(Rc::clone(&p4));
    components
        .hash_packages
        .insert(p1_fmri.clone().get_package_name_as_string(), Rc::clone(&p1));
    components
        .hash_packages
        .insert(p2_fmri.clone().get_package_name_as_string(), Rc::clone(&p2));
    components
        .hash_packages
        .insert(p3_fmri.clone().get_package_name_as_string(), Rc::clone(&p3));
    components
        .hash_packages
        .insert(p4_fmri.clone().get_package_name_as_string(), Rc::clone(&p4));

    let c = |name: String,
             packages: Vec<Rc<RefCell<Package>>>,
             deps: Vec<Rc<RefCell<Package>>>|
     -> Rc<RefCell<Component>> {
        Rc::new(RefCell::new(Component {
            name,
            packages: packages.iter().map(Rc::downgrade).collect(),
            build: deps.clone().iter().map(Rc::downgrade).collect(),
            test: deps.clone().iter().map(Rc::downgrade).collect(),
            sys_build: deps.clone().iter().map(Rc::downgrade).collect(),
            sys_test: deps.clone().iter().map(Rc::downgrade).collect(),
        }))
    };

    let c1 = c(
        "first/component".to_owned(),
        vec![Rc::clone(&p1), Rc::clone(&p2)],
        vec![Rc::clone(&p3), Rc::clone(&p4)],
    );
    let c2 = c(
        "second/component".to_owned(),
        vec![Rc::clone(&p3), Rc::clone(&p4)],
        vec![Rc::clone(&p1), Rc::clone(&p2)],
    );
    components.components.push(Rc::clone(&c1));
    components.components.push(Rc::clone(&c2));
    components
        .hash_components
        .insert("first/component".to_owned(), Rc::clone(&c1));
    components
        .hash_components
        .insert("second/component".to_owned(), Rc::clone(&c2));

    let cp = |p: Rc<RefCell<Package>>, c: &Rc<RefCell<Component>>, d: &Rc<RefCell<Component>>| {
        let mut p = p.borrow_mut();
        p.component = Some(Rc::clone(c));
        p.build_dependents = vec![Rc::clone(d)];
        p.test_dependents = vec![Rc::clone(d)];
        p.sys_build_dependents = vec![Rc::clone(d)];
        p.sys_test_dependents = vec![Rc::clone(d)];
    };

    cp(p1, &c1, &c2);
    cp(p2, &c1, &c2);
    cp(p3, &c2, &c1);
    cp(p4, &c2, &c1);

    components.distribute_reverse_runtime_dependencies();
    components
}

fn new_data() -> Components {
    let p1_fmri = FMRI::parse_raw("p1").unwrap();
    let p2_fmri = FMRI::parse_raw("p2").unwrap();
    let p3_fmri = FMRI::parse_raw("p3").unwrap();
    let p4_fmri = FMRI::parse_raw("p4").unwrap();
    let mut p1 = Package::new(p1_fmri.clone());
    let mut p2 = Package::new(p2_fmri.clone());
    let mut p3 = Package::new(p3_fmri.clone());
    let mut p4 = Package::new(p4_fmri.clone());
    let mut p1_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p1_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    let mut p2_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p2_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    let mut p3_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p3_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    let mut p4_v1 = PackageVersion::new(Version::new("1".to_owned()).unwrap());
    let mut p4_v2 = PackageVersion::new(Version::new("2".to_owned()).unwrap());
    p1_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p3_fmri.clone())]);
    p1_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p3_fmri.clone())]);
    p2_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p4_fmri.clone())]);
    p2_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p4_fmri.clone())]);
    p3_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p1_fmri.clone())]);
    p3_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p1_fmri.clone())]);
    p4_v1.add_runtime_dependencies(&mut vec![DependTypes::Require(p2_fmri.clone())]);
    p4_v2.add_runtime_dependencies(&mut vec![DependTypes::Require(p2_fmri.clone())]);
    p1_v2.set_obsolete(true);
    p1_v1.set_renamed(true);
    p2_v1.set_obsolete(true);
    p2_v2.set_renamed(true);
    p1.add_package_version(p1_v1).unwrap();
    p1.add_package_version(p1_v2).unwrap();
    p2.add_package_version(p2_v1).unwrap();
    p2.add_package_version(p2_v2).unwrap();
    p3.add_package_version(p3_v1).unwrap();
    p3.add_package_version(p3_v2).unwrap();
    p4.add_package_version(p4_v1).unwrap();
    p4.add_package_version(p4_v2).unwrap();
    let mut components = Components::default();
    components.add_package(p1);
    components.add_package(p2);
    components.add_package(p3);
    components.add_package(p4);
    components
        .new_component(
            "first/component".to_owned(),
            vec![p1_fmri.clone(), p2_fmri.clone()],
        )
        .unwrap();
    components
        .new_component(
            "second/component".to_owned(),
            vec![p3_fmri.clone(), p4_fmri.clone()],
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"first/component".to_owned(),
            vec![p3_fmri.clone(), p4_fmri.clone()],
            &DependencyTypes::Build,
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"first/component".to_owned(),
            vec![p3_fmri.clone(), p4_fmri.clone()],
            &DependencyTypes::Test,
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"first/component".to_owned(),
            vec![p3_fmri.clone(), p4_fmri.clone()],
            &DependencyTypes::SystemBuild,
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"first/component".to_owned(),
            vec![p3_fmri.clone(), p4_fmri.clone()],
            &DependencyTypes::SystemTest,
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"second/component".to_owned(),
            vec![p1_fmri.clone(), p2_fmri.clone()],
            &DependencyTypes::Build,
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"second/component".to_owned(),
            vec![p1_fmri.clone(), p2_fmri.clone()],
            &DependencyTypes::Test,
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"second/component".to_owned(),
            vec![p1_fmri.clone(), p2_fmri.clone()],
            &DependencyTypes::SystemBuild,
        )
        .unwrap();
    components
        .add_repo_dependencies(
            &"second/component".to_owned(),
            vec![p1_fmri.clone(), p2_fmri.clone()],
            &DependencyTypes::SystemTest,
        )
        .unwrap();
    components.distribute_reverse_runtime_dependencies();
    components
}
