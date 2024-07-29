use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use fmri::FMRI;
use log::warn;

use crate::{
    assets::catalogs_c::open_json_file,
    packages::dependency_type::{
        DependencyTypes,
        DependencyTypes::{Build, Runtime, SystemBuild, SystemTest, Test},
    },
    problems::{Problem::UnRunnableMakeCommand, Problems},
    Components,
};

pub fn load_git(components: &mut Components, oi_userland_components: &Path) -> Result<(), String> {
    let components_path = oi_userland_components.to_string_lossy();

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && rm -f components.mk ; gmake COMPONENTS_IGNORE=/dev/null components.mk",
            components_path
        ))
        .output()
        .map_err(|e| format!("failed to run command: {}", e))?;

    if !output.stderr.is_empty() {
        warn!(
            "stderr of command \"sh -c cd {} && \
        rm -f components.mk ; gmake COMPONENTS_IGNORE=/dev/null components.mk\" is not empty:\n{}",
            components_path,
            String::from_utf8(output.stderr)
                .map_err(|e| format!("failed to get String from stderr: {}", e))?
        )
    }

    let mut component_list = String::new();
    File::open(Path::new(&format!("{}/components.mk", components_path)))
        .map_err(|e| {
            format!(
                "failed to open file {}/components.mk: {}",
                components_path, e
            )
        })?
        .read_to_string(&mut component_list)
        .map_err(|e| format!("failed to read to String components.mk: {}", e))?;

    for line in component_list.split('\n') {
        if line.is_empty() {
            continue;
        }

        let component_name = line
            .split_whitespace()
            .last()
            .ok_or(format!("failed to get component name from line: {}", line))?
            .to_owned();
        let component_path = format!("{}/{}", components_path, component_name);

        let mut packages: Vec<FMRI> = Vec::new();
        for fmri in open_json_file(&PathBuf::from(format!("{}/pkg5", component_path)))?
            .as_object()
            .ok_or("expect object")?
            .get("fmris")
            .ok_or("expect fmris")?
            .as_array()
            .ok_or("expect array")?
        {
            packages.push(
                FMRI::parse_raw(fmri.as_str().ok_or("expect string")?)
                    .map_err(|e| format!("failed to parse fmri: {}", e))?,
            );
        }

        components
            .new_component(component_name.clone(), packages)
            .map_err(|e| format!("failed to create new compoennt: {}", e))?;

        let mut add_repo_dependencies = |dependency_type: &DependencyTypes| -> Result<(), String> {
            let dependencies =
                get_git_dependencies(&component_path, &mut components.problems, dependency_type)
                    .map_err(|e| {
                        format!(
                            "failed to get {} dependencies from git: {}",
                            dependency_type, e
                        )
                    })?;

            components
                .add_repo_dependencies(&component_name, dependencies, dependency_type)
                .map_err(|e| {
                    format!(
                        "failed to add {} dependencies into component {}: {}",
                        dependency_type, component_name, e
                    )
                })?;

            Ok(())
        };

        add_repo_dependencies(&Build)?;
        add_repo_dependencies(&Test)?;
        add_repo_dependencies(&SystemBuild)?;
        add_repo_dependencies(&SystemTest)?;
    }

    for component in components.clone().get_components() {
        let a = component.borrow();

        let history_file = format!("{}/{}/history", components_path, a.get_name());
        let history_file_path = Path::new(&history_file);

        if !history_file_path.exists() {
            continue;
        }

        let mut history_content = String::new();
        File::open(history_file_path)
            .map_err(|e| {
                format!(
                    "failed to open history file {}: {}",
                    history_file_path.display(),
                    e
                )
            })?
            .read_to_string(&mut history_content)
            .map_err(|e| format!("failed to read to String components.mk: {}", e))?;

        for line in history_content.split('\n') {
            if line.is_empty() {
                // TODO: add warning?
                continue;
            }

            let history = line.split_whitespace().collect::<Vec<&str>>();
            match (history.len(), history.as_slice()) {
                (1, ["noincorporate"]) => {
                    warn!("in history file ({}) is line with only \"noincorporate\", skipping this line", history_file_path.display());
                    continue;
                }
                (2, [raw_fmri, "noincorporate"]) | (1, [raw_fmri]) => {
                    // obsolete package

                    let fmri = FMRI::parse_raw(raw_fmri)
                        .map_err(|e| format!("failed to parse fmri: {}", e))?;

                    if !fmri.has_version() {
                        todo!()
                    }

                    components
                        .set_package_obsolete(fmri)
                        .map_err(|e| format!("failed to set package obsolete: {}", e))?;
                }
                (3, [raw_fmri, _, "noincorporate"]) | (2, [raw_fmri, _]) => {
                    // renamed package

                    let fmri = FMRI::parse_raw(raw_fmri)
                        .map_err(|e| format!("failed to parse fmri: {}", e))?;

                    if !fmri.has_version() {
                        todo!()
                    }

                    components
                        .set_package_renamed(fmri)
                        .map_err(|e| format!("failed to set package renamed: {}", e))?;
                }
                (l, _) if l > 3 => {
                    warn!(
                        "line in history file ({}) has more than 3 columns, skipping this line",
                        history_file_path.display()
                    );
                    continue;
                }
                (3, _) => {
                    warn!("line in history file ({}) has 3 columns, but 3. is not \"noincorporate\", skipping this line", history_file_path.display());
                    continue;
                }
                _ => {
                    return Err(format!(
                        "can not parse line from history file ({}): {}",
                        history_file_path.display(),
                        line
                    ))
                }
            }
        }
    }

    Ok(())
}

fn get_git_dependencies(
    component_path: &String,
    problems: &mut Problems,
    dependency_type: &DependencyTypes,
) -> Result<Vec<FMRI>, String> {
    let mut make_command: String = "gmake ".to_owned();

    #[cfg(target_os = "linux")]
    make_command.push_str("GSED=/usr/bin/sed ");

    make_command.push_str(match dependency_type {
        Runtime => return Err("can not find runtime dependencies in git".to_string()),
        Build => "print-value-REQUIRED_PACKAGES",
        Test => "print-value-TEST_REQUIRED_PACKAGES",
        SystemBuild => "print-value-USERLAND_REQUIRED_PACKAGES",
        SystemTest => "print-value-USERLAND_TEST_REQUIRED_PACKAGES",
    });

    let command = Command::new("sh")
        .arg("-c")
        .arg(format!("cd {} && {}", component_path, make_command))
        .output()
        .map_err(|e| format!("failed to run command: {}", e))?;

    if command.status.code().ok_or("command terminated")? != 0 {
        problems.add_problem(UnRunnableMakeCommand(
            make_command.to_owned(),
            PathBuf::from(component_path),
        ));
    }

    let mut fmri_list: Vec<FMRI> = Vec::new();
    for raw_fmri in String::from_utf8_lossy(&command.stdout).split_whitespace() {
        fmri_list
            .push(FMRI::parse_raw(raw_fmri).map_err(|e| format!("failed to parse fmri: {}", e))?);
    }

    Ok(fmri_list)
}
