use fmri::FMRI;

use crate::packages::package::Package;
use crate::packages::package_versions::PackageVersions;

#[test]
fn add_package_1() {
    // set config
    let (obsolete1, renamed1) = (false, false);
    let (obsolete2, renamed2) = (false, false);
    let (obsolete3, renamed3) = (false, false);
    let obsolete = false;

    let mut package_versions = PackageVersions::new(
        FMRI::new_from_package_name("test".to_string())
            .unwrap()
            .clone(),
    );
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@1".to_owned()).unwrap(),
        obsolete1,
        renamed1,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@2".to_owned()).unwrap(),
        obsolete2,
        renamed2,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@3".to_owned()).unwrap(),
        obsolete3,
        renamed3,
    ));

    assert_eq!(
        package_versions,
        PackageVersions {
            fmri: FMRI::new_from_package_name("test".to_string()).unwrap(),
            obsolete,
            renamed: false,
            packages: vec![
                // Package::new(FMRI::parse_raw(&"test@1".to_string()), obsolete1, renamed1),
                // Package::new(FMRI::parse_raw(&"test@2".to_string()), obsolete2, renamed2),
                Package::new(
                    FMRI::parse_raw(&"test@3".to_string()).unwrap(),
                    obsolete3,
                    renamed3
                )
            ],
        }
    );
}

#[test]
fn add_package_2() {
    // set config
    let (obsolete1, renamed1) = (false, false);
    let (obsolete2, renamed2) = (false, false);
    let (obsolete3, renamed3) = (true, false);
    let obsolete = true;

    let mut package_versions = PackageVersions::new(
        FMRI::new_from_package_name("test".to_string())
            .unwrap()
            .clone(),
    );
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@1".to_owned()).unwrap(),
        obsolete1,
        renamed1,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@2".to_owned()).unwrap(),
        obsolete2,
        renamed2,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@3".to_owned()).unwrap(),
        obsolete3,
        renamed3,
    ));

    assert_eq!(
        package_versions,
        PackageVersions {
            fmri: FMRI::new_from_package_name("test".to_string()).unwrap(),
            obsolete,
            renamed: false,
            packages: vec![
                // Package::new(FMRI::parse_raw(&"test@1".to_string()), obsolete1, renamed1),
                Package::new(
                    FMRI::parse_raw(&"test@2".to_string()).unwrap(),
                    obsolete2,
                    renamed2
                ),
                // Package::new(FMRI::parse_raw(&"test@3".to_string()), obsolete3, renamed3)
            ],
        }
    );
}

#[test]
fn add_package_3() {
    // set config
    let (obsolete1, renamed1) = (false, false);
    let (obsolete2, renamed2) = (true, false);
    let (obsolete3, renamed3) = (false, false);
    let obsolete = false;

    let mut package_versions = PackageVersions::new(
        FMRI::new_from_package_name("test".to_string())
            .unwrap()
            .clone(),
    );
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@1".to_owned()).unwrap(),
        obsolete1,
        renamed1,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@2".to_owned()).unwrap(),
        obsolete2,
        renamed2,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@3".to_owned()).unwrap(),
        obsolete3,
        renamed3,
    ));

    assert_eq!(
        package_versions,
        PackageVersions {
            fmri: FMRI::new_from_package_name("test".to_string()).unwrap(),
            obsolete,
            renamed: false,
            packages: vec![
                // Package::new(FMRI::parse_raw(&"test@1".to_string()), obsolete1, renamed1),
                // Package::new(FMRI::parse_raw(&"test@2".to_string()), obsolete2, renamed2),
                Package::new(
                    FMRI::parse_raw(&"test@3".to_string()).unwrap(),
                    obsolete3,
                    renamed3
                )
            ],
        }
    );
}

#[test]
fn add_package_4() {
    // set config
    let (obsolete1, renamed1) = (false, false);
    let (obsolete2, renamed2) = (false, true);
    let (obsolete3, renamed3) = (false, false);
    let obsolete = false;

    let mut package_versions = PackageVersions::new(
        FMRI::new_from_package_name("test".to_string())
            .unwrap()
            .clone(),
    );
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@1".to_owned()).unwrap(),
        obsolete1,
        renamed1,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@2".to_owned()).unwrap(),
        obsolete2,
        renamed2,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@3".to_owned()).unwrap(),
        obsolete3,
        renamed3,
    ));

    assert_eq!(
        package_versions,
        PackageVersions {
            fmri: FMRI::new_from_package_name("test".to_string()).unwrap(),
            obsolete,
            renamed: false,
            packages: vec![
                // Package::new(FMRI::parse_raw(&"test@1".to_string()), obsolete1, renamed1),
                // Package::new(FMRI::parse_raw(&"test@2".to_string()), obsolete2, renamed2),
                Package::new(
                    FMRI::parse_raw(&"test@3".to_string()).unwrap(),
                    obsolete3,
                    renamed3
                )
            ],
        }
    );
}

#[test]
fn add_package_5() {
    // set config
    let (obsolete1, renamed1) = (false, false);
    let (obsolete2, renamed2) = (false, false);
    let (obsolete3, renamed3) = (false, true);
    let obsolete = false;

    let mut package_versions = PackageVersions::new(
        FMRI::new_from_package_name("test".to_string())
            .unwrap()
            .clone(),
    );
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@1".to_owned()).unwrap(),
        obsolete1,
        renamed1,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@2".to_owned()).unwrap(),
        obsolete2,
        renamed2,
    ));
    package_versions.add_package(Package::new(
        FMRI::parse_raw(&"test@3".to_owned()).unwrap(),
        obsolete3,
        renamed3,
    ));

    assert_eq!(
        package_versions,
        PackageVersions {
            fmri: FMRI::new_from_package_name("test".to_string()).unwrap(),
            obsolete,
            renamed: true,
            packages: vec![
                // Package::new(FMRI::parse_raw(&"test@1".to_string()), obsolete1, renamed1),
                // Package::new(FMRI::parse_raw(&"test@2".to_string()), obsolete2, renamed2),
                Package::new(
                    FMRI::parse_raw(&"test@3".to_string()).unwrap(),
                    obsolete3,
                    renamed3
                )
            ],
        }
    );
}

#[test]
#[should_panic]
fn add_package_6() {
    // set config
    let (obsolete1, renamed1) = (true, true); // impossible = should panic
    PackageVersions::new(
        FMRI::new_from_package_name("test".to_string())
            .clone()
            .unwrap(),
    )
    .add_package(Package::new(
        FMRI::parse_raw(&"test@1".to_owned()).unwrap(),
        obsolete1,
        renamed1,
    ));
}
