use crate::packages::{
    component::Component, components::Components, depend_types::DependTypes,
    dependencies::Dependencies, dependency::Dependency, package::Package,
    package_versions::PackageVersions,
};
use crate::{problems::Problem::RenamedPackageInComponent, ComponentPackagesList, Problems};
use fmri::{FMRIList, Publisher, Version, FMRI};
use log::{debug, error};
use serde_json::Value;
use std::{env, fs::File, io::Read, path::PathBuf, process::exit};

#[derive(Debug)]
enum Attribute {
    Fmri(FMRI),
    DType(String),
    Name(String),
    Value(String),
    Predicate(FMRI),
    Other(String, String),
}

struct Attributes(Vec<Attribute>);

enum Results {
    Dependency(Box<DependTypes>),
    Obsolete,
    Renamed,
    Other,
}

enum Name {
    Obsolete,
    Renamed,
    Other,
}

impl Attributes {
    /// returns only fmri and type attribute
    fn parse_attributes(attributes_string: String) -> Self {
        let mut attributes = Self(vec![]);

        for attribute_value in attributes_string.split_whitespace() {
            if !attribute_value.contains('=') {
                panic!("expected attribute, but it isn't!!! (attribute has \"=\")")
            }
            let (attribute, value) = attribute_value
                .split_once('=')
                .expect("bad attribute value");
            let att = match attribute {
                "fmri" => Attribute::Fmri(FMRI::parse_raw(&value.to_owned())),
                "type" => Attribute::DType(value.to_owned()),
                "name" => Attribute::Name(value.to_owned()),
                "value" => Attribute::Value(value.to_owned()),
                "predicate" => Attribute::Predicate(FMRI::parse_raw(&value.to_owned())),
                _ => {
                    debug!("Unknown attribute found: {} value: {}", attribute, value);
                    Attribute::Other(attribute.to_owned(), value.to_owned())
                }
            };

            attributes.0.push(att)
        }

        attributes
    }

    fn get_type_from_attributes(&self) -> &String {
        for attribute in self.get() {
            if let Attribute::DType(d_type) = attribute {
                return d_type;
            }
        }
        panic!("in attributes is not type attribute!")
    }

    fn get_name_from_attributes(&self) -> Option<&String> {
        for attribute in self.get() {
            if let Attribute::Name(name) = attribute {
                return Some(name);
            }
        }
        None
    }

    fn get_value_from_attributes(&self) -> &String {
        for attribute in self.get() {
            if let Attribute::Value(value) = attribute {
                return value;
            }
        }
        panic!("in attributes is not value attribute!")
    }

    fn get_fmri_from_attributes(&self) -> FMRI {
        for attribute in self.get() {
            if let Attribute::Fmri(fmri) = attribute {
                return fmri.clone();
            }
        }
        panic!("cant find fmri attribute")
    }

    fn get_predicate_from_attributes(&self) -> FMRI {
        for attribute in self.get() {
            if let Attribute::Predicate(fmri) = attribute {
                return fmri.clone();
            }
        }
        panic!("cant find fmri attribute")
    }

    fn get(&self) -> &Vec<Attribute> {
        &self.0
    }
}

/// Returns only depend actions
/// Parses "depend fmri=pkg:/system/library@0.5.11-2017.0.0.16778 type=require" into [`DependTypes`]
fn parse_depend(depend: String) -> DependTypes {
    if !depend.starts_with("depend") {
        // action is not depend
        panic!("bad function calling")
    }

    let attributes = Attributes::parse_attributes(depend.trim_start_matches("depend").to_owned());

    let d_type = attributes.get_type_from_attributes();

    return match d_type.as_str() {
        "require" => DependTypes::Require(attributes.get_fmri_from_attributes()),
        "optional" => DependTypes::Optional(attributes.get_fmri_from_attributes()),
        "incorporate" => DependTypes::Incorporate(attributes.get_fmri_from_attributes()),
        "require-any" => {
            let mut fmri_list = FMRIList::new();
            for attribute in attributes.get() {
                if let Attribute::Fmri(fmri) = attribute {
                    fmri_list.add(fmri.clone())
                }
            }
            if fmri_list.is_empty() {
                panic!("cant find fmri attribute in require-any depend")
            }
            DependTypes::RequireAny(fmri_list)
        }
        "conditional" => DependTypes::Conditional(
            attributes.get_fmri_from_attributes(),
            attributes.get_predicate_from_attributes(),
        ),
        "group" => DependTypes::Group(attributes.get_fmri_from_attributes()),
        _ => panic!("unknown depend type: {}", d_type),
    };
}

fn parse_set(set: String) -> Name {
    if !set.starts_with("set") {
        // action is not set
        panic!("bad function calling")
    }

    let attributes = Attributes::parse_attributes(set.trim_start_matches("set").to_owned());

    match attributes.get_name_from_attributes() {
        None => panic!("\"name\" is not in attributes"),
        Some(name) => {
            if name == "pkg.obsolete" && attributes.get_value_from_attributes() == "true" {
                return Name::Obsolete;
            }
        }
    }

    match attributes.get_name_from_attributes() {
        None => panic!("\"name\" is not in attributes"),
        Some(name) => {
            if name == "pkg.renamed" && attributes.get_value_from_attributes() == "true" {
                return Name::Renamed;
            }
        }
    }

    Name::Other
}

fn parse_action(action: String) -> Results {
    if action.starts_with("depend") {
        return Results::Dependency(Box::new(parse_depend(action.clone())));
    }

    if action.starts_with("set") {
        return match parse_set(action.clone()) {
            Name::Obsolete => Results::Obsolete,
            Name::Renamed => Results::Renamed,
            Name::Other => Results::Other,
        };
    }

    panic!("other unknown action: {}", &action.clone())
}

pub fn open_json_file(mut source_path: PathBuf) -> Value {
    if !source_path.is_absolute() {
        if let Ok(mut current_dir) = env::current_dir() {
            current_dir.push(source_path);
            source_path = current_dir;
        } else {
            error!("can not get current dir of {:?}", source_path);
            exit(1);
        }
    }

    // open file
    let mut file = File::open(source_path.clone())
        .unwrap_or_else(|_| panic!("failed to open file {:?}", source_path));

    // get content
    let mut contains = String::new();
    file.read_to_string(&mut contains)
        .expect("failed to read file");

    // parse json and return
    match serde_json::from_str::<Value>(&contains) {
        Ok(json) => json,
        Err(err) => {
            error!(
                "fatal invalid JSON found in {:?}, error: {}",
                source_path, err
            );
            exit(1);
        }
    }
}

pub fn load_catalog_c(
    components: &mut Components,
    source_path: PathBuf,
    problems: &mut Problems,
    package_names_in_pkg5_list: &ComponentPackagesList,
) {
    // open json file
    let json_value = open_json_file(source_path);

    // for every publisher(String) nad packages(Object) in json
    for (publisher, packages) in json_value.as_object().expect("expected object") {
        // skip _SIGNATURE
        if publisher == "_SIGNATURE" {
            continue;
        }

        // for package_name(String), package_versions(Object) in packages
        for (package_name, package_versions) in packages.as_object().expect("expected object") {
            // create fmri of package
            let mut fmri = FMRI::parse_raw(package_name);
            fmri.change_publisher(Publisher::new(publisher.clone()));

            // create package_versions with above fmri
            let mut versions = PackageVersions::new(fmri.clone());

            // for package_version(Object) in package_versions(Array)
            for package_version in package_versions.as_array().expect("expected array") {
                // Create dependencies
                let mut dependencies = Dependencies::new();
                let mut obsolete = false;
                let mut renamed = false;

                // for key(String)[actions|version] and value(array|String) in package_version
                for (key, value) in package_version.as_object().expect("expected object") {
                    if key == "actions" {
                        // for action(String) in actions(array)
                        for action in value.as_array().expect("array") {
                            // parse action into dependency
                            match parse_action(action.as_str().expect("str").to_owned()) {
                                Results::Dependency(d_type) => {
                                    dependencies.add(Dependency::new(&d_type));
                                }
                                Results::Obsolete => obsolete = true,
                                Results::Renamed => renamed = true,
                                Results::Other => {}
                            }
                        }
                    } else if key == "version" {
                        // get version of current package_version
                        // it is changing on every package_version (will be used in *)
                        fmri.change_version(Version::new(
                            value.clone().as_str().expect("str").to_string(),
                        ));
                    } else {
                        panic!("unknown key: {}", key)
                    }
                }

                // create package with fmri with version of current package_version (changed in *)
                let mut package = Package::new(fmri.clone(), obsolete, renamed);

                // add dependencies into package
                package.add_runtime_dependencies(dependencies);

                // add package into package_versions
                match versions.add_package(package.clone()) {
                    None => {}
                    Some(_) => {
                        // add obsolete
                        components.add_obsoleted(package.clone().fmri());

                        // TODO: RenamedPackageInComponent is already being collected in get_component_packages_of_package_versions (remove this?)
                        if package.is_obsolete() {
                            for component_packages in package_names_in_pkg5_list.get() {
                                for package_in_pkg5 in
                                    component_packages.packages_in_component.get_ref()
                                {
                                    if package.fmri_ref().get_package_name_as_ref_string()
                                        == package_in_pkg5.get_package_name_as_ref_string()
                                    {
                                        problems.add_problem(RenamedPackageInComponent(
                                            package.clone().fmri(),
                                            component_packages.component_name.clone(),
                                        ));
                                    }
                                }
                            }
                        } else {
                            panic!("function .add_package() can return Some(_) only when obsolete package is entered")
                        }
                    }
                }
            }

            // create new component with only one package_versions
            let mut component = Component::new("".to_owned());
            component.add(versions);

            // add component into components
            components.add(component);
        }
    }
    // remove empty components and package versions
    components.remove_empty_package_versions();
    components.remove_empty_components();
}
