use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

/// Represents
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq)]
pub enum DependencyTypes {
    Runtime,
    Build,
    Test,
    SystemBuild,
    SystemTest,
    None,
}

/// Implementation of [`Display`]
impl Display for DependencyTypes{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DependencyTypes::Runtime => "runtime",
            DependencyTypes::Build => "build",
            DependencyTypes::Test => "test",
            DependencyTypes::SystemBuild => "system-build",
            DependencyTypes::SystemTest => "system-test",
            DependencyTypes::None => "none",
        })
    }
}