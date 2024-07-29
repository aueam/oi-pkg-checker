use std::{fs::File, io::Read, path::PathBuf};

use fmri::{FMRI, FMRIList, Publisher, Version};
use log::debug;
use serde_json::Value;

use crate::{
    assets::catalogs_c::Name::{Obsolete, Other, Renamed},
    packages::{
        components::Components,
        depend_types::DependTypes,
        package::{Package, PackageVersion},
    },
};

/// for loading catalog into Components
pub fn load_catalog_c(components: &mut Components, source_path: &PathBuf) -> Result<(), String> {
    // open json file
    let json_value = open_json_file(source_path)?;

    // for every publisher(String) nad packages(Object) in json
    for (publisher, packages) in json_value.as_object().ok_or("expect object")? {
        // skip _SIGNATURE
        if publisher == "_SIGNATURE" {
            continue;
        }

        let publisher = Publisher::new(publisher.clone())
            .map_err(|e| format!("failed to create publisher ({}): {}", publisher, e))?;

        // for package_name(String), package_versions(Object) in packages
        for (package_name, package_versions) in packages.as_object().ok_or("expect object")? {
            // create fmri of package
            let mut fmri = FMRI::parse_raw(package_name).map_err(|e| {
                format!(
                    "failed to parse fmri from package name ({}): {}",
                    package_name, e
                )
            })?;
            fmri.change_publisher(publisher.clone());

            // create package with fmri
            let mut package = Package::new(fmri.clone());

            // for package_version(Object) in package_versions(Array)
            for package_version in package_versions.as_array().ok_or("expect array")? {
                let mut runtime_dependencies: Vec<DependTypes> = Vec::new();
                let mut obsolete = false;
                let mut renamed = false;

                // for key(String)[actions|version] and value(array|String) in package_version
                for (key, value) in package_version.as_object().ok_or("expect object")? {
                    if key == "actions" {
                        // get actions

                        // for action(String) in actions(array)
                        for action in value.as_array().ok_or("expect array")? {
                            // parse action into dependency
                            match parse_action(action.as_str().ok_or("expect str")?.to_owned())
                                .map_err(|e| {
                                    format!("failed to parse action ({}): {}", action, e)
                                })? {
                                ParsedAction::Dependency(d_type) => {
                                    runtime_dependencies.push(*d_type);
                                }
                                ParsedAction::Obsolete => obsolete = true,
                                ParsedAction::Renamed => renamed = true,
                                ParsedAction::Other => {}
                            }
                        }
                    } else if key == "version" {
                        // get version

                        // create package version with version
                        let mut package_version = PackageVersion::new(
                            Version::new(value.clone().as_str().ok_or("expect str")?.to_string())
                                .map_err(|e| format!("failed to parse version: {}", e))?,
                        );
                        package_version.add_runtime_dependencies(&mut runtime_dependencies);
                        package_version.set_obsolete(obsolete);
                        package_version.set_renamed(renamed);

                        package.add_package_version(package_version).map_err(|e| {
                            format!("failed to add package version into package: {}", e)
                        })?;
                    } else {
                        return Err(format!("unknown key: {} (expect version or actions)", key));
                    }
                }
            }

            // add new package
            components.add_package(package);
        }
    }

    components.remove_old_versions();
    components.distribute_reverse_runtime_dependencies();

    Ok(())
}

enum ParsedAction {
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

fn parse_action(action: String) -> Result<ParsedAction, String> {
    if action.starts_with("depend") {
        return Ok(ParsedAction::Dependency(Box::new(
            parse_depend(action.clone())
                .map_err(|e| format!("failed to parse depend action: {}", e))?,
        )));
    }

    if action.starts_with("set") {
        return Ok(
            match parse_set(action.clone())
                .map_err(|e| format!("failed to parse set action: {}", e))?
            {
                Obsolete => ParsedAction::Obsolete,
                Renamed => ParsedAction::Renamed,
                Other => ParsedAction::Other,
            },
        );
    }

    Err(format!("other unknown action: {}", &action.clone()))
}

enum Attribute {
    Fmri(FMRI),
    DType(String),
    Name(String),
    Value(String),
    Predicate(FMRI),
    Other,
}

struct Attributes(Vec<Attribute>);

impl Attributes {
    /// returns only fmri and type attribute
    fn parse_attributes(attributes_string: String) -> Result<Self, String> {
        let mut attributes = Self(vec![]);

        for attribute_value in attributes_string.split_whitespace() {
            if !attribute_value.contains('=') {
                return Err("expected attribute, but it isn't! (attribute has \"=\")".to_owned());
            }
            let (attribute, value) = attribute_value
                .split_once('=')
                .ok_or("bad attribute value".to_string())?;
            let att = match attribute {
                "fmri" => Attribute::Fmri(
                    FMRI::parse_raw(value).map_err(|e| format!("failed to parse fmri: {}", e))?,
                ),
                "type" => Attribute::DType(value.to_owned()),
                "name" => Attribute::Name(value.to_owned()),
                "value" => Attribute::Value(value.to_owned()),
                "predicate" => Attribute::Predicate(
                    FMRI::parse_raw(value).map_err(|e| format!("failed to parse fmri: {}", e))?,
                ),
                _ => {
                    debug!("Unknown attribute found: {} value: {}", attribute, value);
                    Attribute::Other
                }
            };

            attributes.0.push(att)
        }

        Ok(attributes)
    }

    fn get_type_from_attributes(&self) -> Option<&String> {
        for attribute in &self.0 {
            if let Attribute::DType(d_type) = attribute {
                return Some(d_type);
            }
        }
        None
    }

    fn get_name_from_attributes(&self) -> Option<&String> {
        for attribute in &self.0 {
            if let Attribute::Name(name) = attribute {
                return Some(name);
            }
        }
        None
    }

    fn get_value_from_attributes(&self) -> Option<&String> {
        for attribute in &self.0 {
            if let Attribute::Value(value) = attribute {
                return Some(value);
            }
        }
        None
    }

    fn get_fmri_from_attributes(&self) -> Option<FMRI> {
        for attribute in &self.0 {
            if let Attribute::Fmri(fmri) = attribute {
                return Some(fmri.clone());
            }
        }
        None
    }

    fn get_predicate_from_attributes(&self) -> Option<FMRI> {
        for attribute in &self.0 {
            if let Attribute::Predicate(fmri) = attribute {
                return Some(fmri.clone());
            }
        }
        None
    }
}

/// parses set action
fn parse_set(set: String) -> Result<Name, String> {
    if !set.starts_with("set") {
        return Err("trying to parse set action, but it is not set".to_owned());
    }

    let attributes = Attributes::parse_attributes(set.trim_start_matches("set").to_owned())
        .map_err(|e| format!("failed to parse attributes: {}", e))?;

    if attributes
        .get_name_from_attributes()
        .ok_or("failed to get name attribute")?
        == "pkg.obsolete"
        && attributes
            .get_value_from_attributes()
            .ok_or("failed to get value attribute")?
            == "true"
    {
        return Ok(Obsolete);
    }

    if attributes
        .get_name_from_attributes()
        .ok_or("failed to get name attribute")?
        == "pkg.renamed"
        && attributes
            .get_value_from_attributes()
            .ok_or("failed to get value attribute")?
            == "true"
    {
        return Ok(Renamed);
    }

    Ok(Other)
}

/// Returns only depend actions
/// Parses "depend fmri=pkg:/system/library@0.5.11-2017.0.0.16778 type=require" into [`DependTypes`]
fn parse_depend(depend: String) -> Result<DependTypes, String> {
    if !depend.starts_with("depend") {
        return Err("trying to parse depend action, but it is not depend".to_owned());
    }

    let attributes = Attributes::parse_attributes(depend.trim_start_matches("depend").to_owned())
        .map_err(|e| format!("failed to parse attributes: {}", e))?;

    let d_type = attributes
        .get_type_from_attributes()
        .ok_or("failed to get type attribute")?;

    return Ok(match d_type.as_str() {
        "require" => DependTypes::Require(
            attributes
                .get_fmri_from_attributes()
                .ok_or("failed to get fmri attribute")?,
        ),
        "optional" => DependTypes::Optional(
            attributes
                .get_fmri_from_attributes()
                .ok_or("failed to get fmri attribute")?,
        ),
        "incorporate" => DependTypes::Incorporate(
            attributes
                .get_fmri_from_attributes()
                .ok_or("failed to get fmri attribute")?,
        ),
        "require-any" => {
            let mut fmri_list = FMRIList::new();
            for attribute in attributes.0 {
                if let Attribute::Fmri(fmri) = attribute {
                    fmri_list.add(fmri)
                }
            }
            if fmri_list.is_empty() {
                return Err("cant find fmri attribute in require-any depend".to_owned());
            }
            DependTypes::RequireAny(fmri_list)
        }
        "conditional" => DependTypes::Conditional(
            attributes
                .get_fmri_from_attributes()
                .ok_or("failed to get fmri attribute")?,
            attributes
                .get_predicate_from_attributes()
                .ok_or("failed to get predicate attribute")?,
        ),
        "group" => DependTypes::Group(
            attributes
                .get_fmri_from_attributes()
                .ok_or("failed to get fmri attribute")?,
        ),
        _ => return Err(format!("unknown depend type: {}", d_type)),
    });
}

pub fn open_json_file(source_path: &PathBuf) -> Result<Value, String> {
    let mut contains = String::new();
    File::open(source_path.clone())
        .map_err(|e| format!("failed to open file {}: {}", source_path.display(), e))?
        .read_to_string(&mut contains)
        .map_err(|e| format!("failed to read file {}: {}", source_path.display(), e))?;

    serde_json::from_str::<Value>(&contains)
        .map_err(|e| format!("invalid JSON found in {:?}: {}", source_path, e))
}
