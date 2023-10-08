use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use fmri::FMRI;
use fmri::fmri_list::FMRIList;

/// Represents depend action type
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum DependTypes {
    Require(FMRI),
    Optional(FMRI),
    Exclude(FMRI),
    Incorporate(FMRI),
    RequireAny(FMRIList),
    GroupAny(FMRIList),
    Conditional(FMRI, FMRI),
    Origin(FMRI),
    Group(FMRI),
    Parent(FMRI)
}

impl DependTypes {
    /// Returns name of depend action and content
    pub fn get_name_and_content_as_string(self) -> (String, String) {
        match self {
            DependTypes::Require(fmri) => ("require".to_owned(), fmri.get_package_name_as_string()),
            DependTypes::Optional(fmri) => ("optional".to_owned(), fmri.get_package_name_as_string()),
            DependTypes::Incorporate(fmri) => ("incorporate".to_owned(), fmri.get_package_name_as_string()),
            DependTypes::RequireAny(fmri_list) => {
                let mut string = String::new();
                let len = fmri_list.get_ref().len();
                for (index, fmri) in fmri_list.get_ref().iter().enumerate() {
                    string.push_str(fmri.get_package_name_as_ref_string());
                    if index+1 < len {
                        string.push_str(", ");
                    }
                }
                ("require-any".to_owned(), string)
            },
            DependTypes::Conditional(fmri, predicate) => {

                // TODO: remove this
                if fmri.package_name_eq(&FMRI::parse_raw(&"none".to_owned())) {
                    return ("conditional".to_owned(), format!("predicate={}", predicate))
                }
                if predicate.package_name_eq(&FMRI::parse_raw(&"none".to_owned())) {
                    return ("conditional".to_owned(), format!("fmri={}", fmri))
                }

                ("conditional".to_owned(), format!("fmri={}, predicate={}", fmri, predicate))
            },
            DependTypes::Group(fmri) => ("group".to_owned(), fmri.get_package_name_as_string()),
            _ => unimplemented!()
        }
    }

    pub fn get_content_ref(&self) -> Result<&FMRI, &FMRIList> {
        match self {
            DependTypes::Require(fmri) => Ok(fmri),
            DependTypes::Optional(fmri) => Ok(fmri),
            DependTypes::Incorporate(fmri) => Ok(fmri),
            DependTypes::RequireAny(fmri_list) => Err(fmri_list),
            DependTypes::Conditional(fmri, _) => Ok(fmri),
            DependTypes::Group(fmri) => Ok(fmri),
            _ => unimplemented!()
        }
    }
}

/// Implementation of [`Display`]
impl Display for DependTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string: String = "".to_owned();

        match self {
            DependTypes::Require(fmri) => string.push_str(&format!("fmri={} type=require", fmri)),
            DependTypes::Optional(fmri) => string.push_str(&format!("fmri={} type=optional", fmri)),
            DependTypes::Incorporate(fmri) => string.push_str(&format!("fmri={} type=incorporate", fmri)),
            DependTypes::RequireAny(fmri_list) => {
                let mut tmp: String = "".to_owned();
                for fmri in fmri_list.get_ref() {
                    tmp.push_str(&format!("fmri={} ", fmri))
                }
                tmp.push_str("type=require-any");
                string.push_str(&tmp);
            }
            DependTypes::Conditional(fmri, predicate) => string.push_str(&format!("fmri={} predicate={} type=conditional", fmri, predicate)),
            DependTypes::Group(fmri) => string.push_str(&format!("fmri={} type=group", fmri)),
            _ => unimplemented!()
        }

        write!(f, "{}", string)
    }
}