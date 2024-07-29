use crate::packages::rev_depend_type::RevDependType::*;
use fmri::FMRI;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Reverse depend types
///
/// `FMRI` is package that has this runtime dependency on package where `RevDependType` is.
///
/// # Example
///
/// If we have `Require("pkg:/vim")` in package `"pkg:/library/libc"`, it means
/// that package `"pkg:/vim"` has require dependency on package `"pkg:/library/libc"`.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum RevDependType {
    Require(FMRI),
    Optional(FMRI),
    Incorporate(FMRI),
    RequireAny(FMRI),
    ConditionalFmri(FMRI),
    ConditionalPredicate(FMRI),
    Group(FMRI),
}

impl Display for RevDependType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "package={}",
            match self {
                Require(f) => format!("{} type=require", f),
                Optional(f) => format!("{} type=optional", f),
                Incorporate(f) => format!("{} type=incorporate", f),
                RequireAny(f) => format!("{} type=require-any", f),
                ConditionalFmri(f) => format!("{} type=conditional(fmri)", f),
                ConditionalPredicate(f) => format!("{} type=conditional(predicate)", f),
                Group(f) => format!("{} type=group", f),
            }
        )
    }
}
