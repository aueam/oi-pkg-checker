use fmri::FMRI;
use fmri::fmri_list::FMRIList;
use crate::packages::component::Component;
use crate::packages::components::Components;
use crate::packages::dependencies::Dependencies;
use crate::packages::dependency::Dependency;
use crate::packages::depend_types::DependTypes;
use crate::packages::package::Package;
use crate::packages::package_versions::PackageVersions;

#[test]
fn check_useless_packages() {
    let mut components = Components::new();
    let mut component = Component::new("".to_owned());
    let mut package_versions = PackageVersions::new(FMRI::parse_raw(&"test".to_owned()));
    let mut package = Package::new(FMRI::parse_raw(&"test@1".to_owned()), false, false);
    let mut dependencies = Dependencies::new();
    dependencies.add(Dependency::new(&mut DependTypes::Require(FMRI::parse_raw(&"another@1".to_owned()))));
    package.add_runtime_dependencies(dependencies);
    package_versions.add_package(package);
    component.add(package_versions);

    let mut package_versions = PackageVersions::new(FMRI::parse_raw(&"another".to_owned()));
    package_versions.add_package(Package::new(FMRI::parse_raw(&"another@2".to_owned()), false, false));
    package_versions.add_package(Package::new(FMRI::parse_raw(&"another@3".to_owned()), false, false));
    component.add(package_versions);

    components.add(component);


    let mut fmri_list = FMRIList::new();
    fmri_list.add(FMRI::parse_raw(&"test@1".to_owned()));
    assert_eq!(
        components.check_useless_packages(),
        fmri_list
    )
}

// OLD
// #[test]
// fn remove_obsolete_packages() {
//     let mut components = Components::new();
//     let mut component = Component::new("".to_owned());
//     let mut package_versions = PackageVersions::new(FMRI::parse_raw(&"test".to_owned()));
//     package_versions.add_package(Package::new(FMRI::parse_raw(&"test@1".to_owned()), true, false));
//     component.add(package_versions);
//     components.add(component);
//
//     components.remove_obsolete_packages();
//
//     assert_eq!(
//         components,
//         Components::new()
//     );
// }