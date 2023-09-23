use std::path::PathBuf;
use crate::assets::catalog_encumbered_dependency_c::load_encumbered_catalog_dependency_c;
use crate::ComponentPackagesList;
use crate::packages::components::Components;
use crate::problems::RenamedPackageInComponentList;

/// loads catalog into [`Components`]
pub fn load_catalog_dependency_c(components: &mut Components, source_path: PathBuf, package_names_in_pkg5_list: &ComponentPackagesList) -> RenamedPackageInComponentList {
    load_encumbered_catalog_dependency_c(components, source_path, package_names_in_pkg5_list)
}