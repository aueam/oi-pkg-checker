use std::path::PathBuf;

/// All available assets
pub enum Assets {
    Catalogs(Vec<PathBuf>),
    OpenIndianaOiUserlandGit {
        load_component_list: bool,
        load_build_dependencies: bool,
        load_test_dependencies: bool,
        load_system_build_dependencies: bool,
        load_system_test_dependencies: bool
    }
}