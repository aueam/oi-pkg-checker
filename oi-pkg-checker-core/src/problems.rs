use std::fs::File;
use std::io::{Read, Write};
use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use log::{error, info, warn};
use fmri::FMRI;
use bincode::{serialize, deserialize};
use serde::{Deserialize, Serialize};
use crate::packages::components::Components;
use crate::packages::depend_types::DependTypes;
use crate::packages::dependency_type::DependencyTypes;

#[derive(Serialize, Deserialize)]
pub struct Problems {
    missing_component_for_package_list: MissingComponentForPackageList,
    renamed_needs_renamed_list: RenamedNeedsRenamedList,
    renamed_package_in_component_list: RenamedPackageInComponentList,
    un_runnable_make_command_list: UnRunnableMakeCommandList,
    non_existing_required_package_list: NonExistingRequiredPackageList,
    obsolete_required_package_list: ObsoletedRequiredPackageList,
    partly_obsolete_required_package_list: PartlyObsoletedRequiredPackageList,
    useless_components_list: UselessComponentsList,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct MissingComponentForPackage(FMRI);

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct RenamedNeedsRenamed(FMRI, FMRI);

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct RenamedPackageInComponent(FMRI, bool, String);

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct UnRunnableMakeCommand(String, PathBuf);

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct NonExistingRequiredPackage(DependTypes, DependencyTypes, FMRI, bool);

/// for obsolete packages
#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct ObsoletedRequiredPackage(DependTypes, DependencyTypes, FMRI, bool);

/// for obsolete packages with normal older version
#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct PartlyObsoletedRequiredPackage(DependTypes, DependencyTypes, FMRI, bool);

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct UselessComponents(String);


#[derive(Serialize, Deserialize, Clone)]
pub struct MissingComponentForPackageList(Vec<MissingComponentForPackage>);

#[derive(Serialize, Deserialize, Clone)]
pub struct RenamedNeedsRenamedList(Vec<RenamedNeedsRenamed>);

#[derive(Serialize, Deserialize, Clone)]
pub struct RenamedPackageInComponentList(Vec<RenamedPackageInComponent>);

#[derive(Serialize, Deserialize, Clone)]
pub struct UnRunnableMakeCommandList(Vec<UnRunnableMakeCommand>);

#[derive(Serialize, Deserialize, Clone)]
pub struct NonExistingRequiredPackageList(Vec<NonExistingRequiredPackage>);

#[derive(Serialize, Deserialize, Clone)]
pub struct ObsoletedRequiredPackageList(Vec<ObsoletedRequiredPackage>);

#[derive(Serialize, Deserialize, Clone)]
pub struct PartlyObsoletedRequiredPackageList(Vec<PartlyObsoletedRequiredPackage>);

#[derive(Serialize, Deserialize, Clone)]
pub struct UselessComponentsList(Vec<UselessComponents>);

impl Problems {
    pub fn new(
        missing_component_for_package_list: MissingComponentForPackageList,
        renamed_needs_renamed_list: RenamedNeedsRenamedList,
        renamed_package_in_component_list: RenamedPackageInComponentList,
        un_runnable_make_command_list: UnRunnableMakeCommandList,
        non_existing_required_package_list: NonExistingRequiredPackageList,
        obsolete_required_package_list: ObsoletedRequiredPackageList,
        partly_obsolete_required_package_list: PartlyObsoletedRequiredPackageList,
        useless_components_list: UselessComponentsList,
    ) -> Self {
        Self {
            missing_component_for_package_list,
            renamed_needs_renamed_list,
            renamed_package_in_component_list,
            un_runnable_make_command_list,
            non_existing_required_package_list,
            obsolete_required_package_list,
            partly_obsolete_required_package_list,
            useless_components_list,
        }
    }

    pub fn get(self) -> (
        MissingComponentForPackageList,
        RenamedNeedsRenamedList,
        RenamedPackageInComponentList,
        UnRunnableMakeCommandList,
        NonExistingRequiredPackageList,
        ObsoletedRequiredPackageList,
        PartlyObsoletedRequiredPackageList,
        UselessComponentsList
    ) {
        (
            self.missing_component_for_package_list,
            self.renamed_needs_renamed_list,
            self.renamed_package_in_component_list,
            self.un_runnable_make_command_list,
            self.non_existing_required_package_list,
            self.obsolete_required_package_list,
            self.partly_obsolete_required_package_list,
            self.useless_components_list
        )
    }

    pub fn serialize<P: AsRef<Path> + ?Sized>(&self, path: &P) {
        File::create(path)
            .unwrap()
            .write_all(
                &serialize(self)
                    .expect("failed to serialize file with problems into binary")
            )
            .expect("TODO: panic message");
    }

    pub fn deserialize<P: AsRef<Path> + ?Sized>(path: &P) -> Self {
        let data = &mut Vec::new();
        File::open(path)
            .expect("failed to open file with problems")
            .read_to_end(data)
            .expect("failed to read data");
        deserialize(data).expect("failed to deserialize data from binary")
    }
}

impl MissingComponentForPackage {
    pub fn new(package: FMRI) -> Self {
        Self(package)
    }

    pub fn get_package_ref(&self) -> &FMRI {
        &self.0
    }
}

impl RenamedNeedsRenamed {
    pub fn new(package: FMRI, needs_package: FMRI) -> Self {
        Self(package, needs_package)
    }

    pub fn get_package_ref(&self) -> &FMRI {
        &self.0
    }

    pub fn get_needed_package_ref(&self) -> &FMRI {
        &self.1
    }
}

impl RenamedPackageInComponent {
    pub fn new(package: FMRI, renamed_or_obsoleted: bool, component: String) -> Self {
        Self(package, renamed_or_obsoleted, component)
    }

    pub fn get_package_ref(&self) -> &FMRI {
        &self.0
    }

    pub fn get_renamed_or_obsoleted_ref(&self) -> &bool {
        &self.1
    }

    pub fn get_component_ref(&self) -> &String {
        &self.2
    }
}

impl UnRunnableMakeCommand {
    pub fn new(command: String, path: PathBuf) -> Self {
        Self(command, path)
    }

    pub fn get_command_ref(&self) -> &String {
        &self.0
    }

    pub fn get_path_ref(&self) -> &PathBuf {
        &self.1
    }
}

impl NonExistingRequiredPackage {
    pub fn new(depend_type: DependTypes, dependency_type: DependencyTypes, required_by: FMRI, renamed: bool) -> Self {
        Self(depend_type, dependency_type, required_by, renamed)
    }

    pub fn get_depend_type_ref(&self) -> &DependTypes {
        &self.0
    }

    pub fn get_dependency_type_ref(&self) -> &DependencyTypes {
        &self.1
    }

    pub fn get_required_by_ref(&self) -> &FMRI {
        &self.2
    }

    pub fn get_renamed(&self) -> &bool {
        &self.3
    }
}

impl ObsoletedRequiredPackage {
    pub fn new(depend_type: DependTypes, dependency_type: DependencyTypes, required_by: FMRI, renamed: bool) -> Self {
        Self(depend_type, dependency_type, required_by, renamed)
    }

    pub fn get_depend_type_ref(&self) -> &DependTypes {
        &self.0
    }

    pub fn get_dependency_type_ref(&self) -> &DependencyTypes {
        &self.1
    }

    pub fn get_required_by_ref(&self) -> &FMRI {
        &self.2
    }

    pub fn get_renamed(&self) -> &bool {
        &self.3
    }
}

impl PartlyObsoletedRequiredPackage {
    pub fn new(depend_type: DependTypes, dependency_type: DependencyTypes, required_by: FMRI, renamed: bool) -> Self {
        Self(depend_type, dependency_type, required_by, renamed)
    }

    pub fn get_depend_type_ref(&self) -> &DependTypes {
        &self.0
    }

    pub fn get_dependency_type_ref(&self) -> &DependencyTypes {
        &self.1
    }

    pub fn get_required_by_ref(&self) -> &FMRI {
        &self.2
    }

    pub fn get_renamed(&self) -> &bool {
        &self.3
    }
}

impl UselessComponents {
    pub fn new(component_name: String) -> Self {
        Self(component_name)
    }

    pub fn get_ref(&self) -> &String {
        &self.0
    }
}


impl UselessComponentsList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn report(&self) {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        for error in self.get_ref() {
            info!("component {} is not needed by any package", error.get_ref())
        }
    }
}

impl RenamedNeedsRenamedList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn report(&self) {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        for error in self.get_ref() {
            error!(
                "renamed package {} needs renamed package {}",
                error.get_package_ref().get_package_name_as_ref_string(),
                error.get_needed_package_ref().get_package_name_as_ref_string()
            )
        }
    }
}

impl RenamedPackageInComponentList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn report(&self) -> usize {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        let mut printed = Vec::new();
        for error in self.get_ref() {
            if printed.contains(error.get_package_ref().get_package_name_as_ref_string()) {
                continue;
            }

            printed.push(error.get_package_ref().get_package_name_as_ref_string().clone());

            if *error.get_renamed_or_obsoleted_ref() {
                error!("package {} is renamed and is in component {}", error.get_package_ref().get_package_name_as_ref_string(), error.get_component_ref())
            } else {
                error!("package {} is obsolete and is in component {}", error.get_package_ref().get_package_name_as_ref_string(), error.get_component_ref())
            }
        }

        printed.len()
    }
}

impl NonExistingRequiredPackageList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn set_dependency_type(&mut self, dependency_type: DependencyTypes) {
        for non_existing_required_package in self.get_ref_mut() {
            non_existing_required_package.1 = dependency_type.clone()
        }
    }

    fn report(&mut self, components: &Components) -> usize {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        let mut printed = Vec::new();
        for error in self.get_ref_mut() {
            let (name, fmri) = error.get_depend_type_ref().clone().get_name_and_content_as_string();
            let dependency_type = error.get_dependency_type_ref();
            let required_by = error.get_required_by_ref();

            let mut package_or_component_name = match components.get_component_name_by_package(required_by) {
                None => todo!("add panic here"),
                Some(component_name) => "component ".to_owned() + component_name
            };

            let mut by_and_types = match error.get_renamed() {
                true => {
                    package_or_component_name = required_by.get_package_name_as_ref_string().clone();
                    "by renamed package ".to_owned()
                }
                false => {
                    if dependency_type == &DependencyTypes::Runtime {
                        package_or_component_name = required_by.get_package_name_as_ref_string().clone();
                    }
                    "by ".to_owned()
                }
            };

            by_and_types.push_str(
                &match error.get_renamed() {
                    true => match dependency_type {
                        DependencyTypes::Runtime => format!("{} (runtime, {})", package_or_component_name, name),
                        DependencyTypes::Build => format!("{} (build, {})", package_or_component_name, name),
                        DependencyTypes::Test => format!("{} (test, {})", package_or_component_name, name),
                        DependencyTypes::SystemBuild => format!("{} (system-build)", package_or_component_name),
                        DependencyTypes::SystemTest => format!("{} (system-test)", package_or_component_name),
                        DependencyTypes::None => panic!("DependencyTypes can't be None")
                    },
                    false => match dependency_type {
                        DependencyTypes::Runtime => format!("package {} (runtime, {})", package_or_component_name, name),
                        DependencyTypes::Build => format!("{} (build, component)", package_or_component_name),
                        DependencyTypes::Test => format!("{} (test, component)", package_or_component_name),
                        DependencyTypes::SystemBuild => format!("{} (build, system)", package_or_component_name),
                        DependencyTypes::SystemTest => format!("{} (test, system)", package_or_component_name),
                        DependencyTypes::None => panic!("DependencyTypes can't be None")
                    }
                }
            );

            if printed.contains(&format!("{}{}", fmri.clone(), by_and_types.clone())) {
                continue;
            }

            printed.push(format!("{}{}", fmri.clone(), by_and_types.clone()));

            warn!("package {} doesn't exist, but is required {}", fmri, by_and_types)
        }
        printed.len()
    }
}

impl ObsoletedRequiredPackageList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn set_dependency_type(&mut self, dependency_type: DependencyTypes) {
        for obsolete_required_package in self.get_ref_mut() {
            obsolete_required_package.1 = dependency_type.clone()
        }
    }

    fn report(&mut self, components: &Components) -> usize {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        let mut printed = Vec::new();
        for error in self.get_ref_mut() {
            let (name, fmri) = error.get_depend_type_ref().clone().get_name_and_content_as_string();
            let dependency_type = error.get_dependency_type_ref();
            let required_by = error.get_required_by_ref();

            let mut package_or_component_name = match components.get_component_name_by_package(required_by) {
                None => todo!("add panic here"),
                Some(component_name) => "component ".to_owned() + component_name
            };

            let mut by_and_types = match error.get_renamed() {
                true => {
                    package_or_component_name = required_by.get_package_name_as_ref_string().clone();
                    "by renamed package ".to_owned()
                }
                false => {
                    if dependency_type == &DependencyTypes::Runtime {
                        package_or_component_name = required_by.get_package_name_as_ref_string().clone();
                    }
                    "by ".to_owned()
                }
            };

            by_and_types.push_str(
                &match error.get_renamed() {
                    true => match dependency_type {
                        DependencyTypes::Runtime => format!("{} (runtime, {})", package_or_component_name, name),
                        DependencyTypes::Build => format!("{} (build, component)", package_or_component_name),
                        DependencyTypes::Test => format!("{} (test, component)", package_or_component_name),
                        DependencyTypes::SystemBuild => format!("{} (system-build, system)", package_or_component_name),
                        DependencyTypes::SystemTest => format!("{} (system-test, system)", package_or_component_name),
                        DependencyTypes::None => panic!("DependencyTypes can't be None")
                    },
                    false => match dependency_type {
                        DependencyTypes::Runtime => format!("package {} (runtime, {})", package_or_component_name, name),
                        DependencyTypes::Build => format!("{} (build, component)", package_or_component_name),
                        DependencyTypes::Test => format!("{} (test, component)", package_or_component_name),
                        DependencyTypes::SystemBuild => format!("{} (build, system)", package_or_component_name),
                        DependencyTypes::SystemTest => format!("{} (test, system)", package_or_component_name),
                        DependencyTypes::None => panic!("DependencyTypes can't be None")
                    }
                }
            );

            if printed.contains(&format!("{}{}", fmri.clone(), by_and_types.clone())) {
                continue;
            }

            printed.push(format!("{}{}", fmri.clone(), by_and_types.clone()));

            error!("obsoleted package {} is required {}", fmri, by_and_types);
        }
        printed.len()
    }
}

impl PartlyObsoletedRequiredPackageList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn set_dependency_type(&mut self, dependency_type: DependencyTypes) {
        for obsolete_required_package in self.get_ref_mut() {
            obsolete_required_package.1 = dependency_type.clone()
        }
    }

    fn report(&mut self, components: &Components) -> usize {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        let mut printed = Vec::new();
        for error in self.get_ref_mut() {
            let (name, fmri) = error.get_depend_type_ref().clone().get_name_and_content_as_string();
            let dependency_type = error.get_dependency_type_ref();
            let required_by = error.get_required_by_ref();

            let mut package_or_component_name = match components.get_component_name_by_package(required_by) {
                None => todo!("add panic here"),
                Some(component_name) => "component ".to_owned() + component_name
            };

            let mut by_and_types = match error.get_renamed() {
                true => {
                    package_or_component_name = required_by.get_package_name_as_ref_string().clone();
                    "by renamed package ".to_owned()
                }
                false => {
                    if dependency_type == &DependencyTypes::Runtime {
                        package_or_component_name = required_by.get_package_name_as_ref_string().clone();
                    }
                    "by ".to_owned()
                }
            };

            by_and_types.push_str(
                &match error.get_renamed() {
                    true => match dependency_type {
                        DependencyTypes::Runtime => format!("{} (runtime, {})", package_or_component_name, name),
                        DependencyTypes::Build => format!("{} (build, component)", package_or_component_name),
                        DependencyTypes::Test => format!("{} (test, component)", package_or_component_name),
                        DependencyTypes::SystemBuild => format!("{} (system-build, system)", package_or_component_name),
                        DependencyTypes::SystemTest => format!("{} (system-test, system)", package_or_component_name),
                        DependencyTypes::None => panic!("DependencyTypes can't be None")
                    },
                    false => match dependency_type {
                        DependencyTypes::Runtime => format!("package {} (runtime, {})", package_or_component_name, name),
                        DependencyTypes::Build => format!("{} (build, component)", package_or_component_name),
                        DependencyTypes::Test => format!("{} (test, component)", package_or_component_name),
                        DependencyTypes::SystemBuild => format!("{} (build, system)", package_or_component_name),
                        DependencyTypes::SystemTest => format!("{} (test, system)", package_or_component_name),
                        DependencyTypes::None => panic!("DependencyTypes can't be None")
                    }
                }
            );

            if printed.contains(&format!("{}{}", fmri.clone(), by_and_types.clone())) {
                continue;
            }

            printed.push(format!("{}{}", fmri.clone(), by_and_types.clone()));

            warn!("obsoleted package {} is required{}", fmri, by_and_types);
        }

        printed.len()
    }
}

impl UnRunnableMakeCommandList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn report(&self) {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        let mut printed = Vec::new();
        for error in self.get_ref() {

            if printed.contains(&format!("{}{:?}", error.get_command_ref(), error.get_path_ref())) {
                continue;
            }
            printed.push(format!("{}{:?}", error.get_command_ref(), error.get_path_ref()));

            error!("can't run {} in {:?}", error.get_command_ref(), error.get_path_ref())
        }
    }
}


impl MissingComponentForPackageList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn report(&self) {
        if self.is_empty() {
            panic!("reported problem can't be empty")
        }

        let mut printed = Vec::new();
        for error in self.get_ref() {

            if printed.contains(&format!("{}", error.get_package_ref())) {
                continue;
            }
            printed.push(format!("{}", error.get_package_ref()));

            warn!("missing component for {}", error.get_package_ref())
        }
    }
}

impl ProblemList for MissingComponentForPackageList {
    type Problem = MissingComponentForPackage;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}

impl ProblemList for RenamedNeedsRenamedList {
    type Problem = RenamedNeedsRenamed;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}

impl ProblemList for RenamedPackageInComponentList {
    type Problem = RenamedPackageInComponent;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}

impl ProblemList for UnRunnableMakeCommandList {
    type Problem = UnRunnableMakeCommand;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}

impl ProblemList for NonExistingRequiredPackageList {
    type Problem = NonExistingRequiredPackage;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}

impl ProblemList for ObsoletedRequiredPackageList {
    type Problem = ObsoletedRequiredPackage;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}

impl ProblemList for PartlyObsoletedRequiredPackageList {
    type Problem = PartlyObsoletedRequiredPackage;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}

impl ProblemList for UselessComponentsList {
    type Problem = UselessComponents;

    fn get(self) -> Vec<Self::Problem> {
        self.0
    }

    fn get_ref(&self) -> &Vec<Self::Problem> {
        &self.0
    }

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem> {
        &mut self.0
    }

    fn is_empty(&self) -> bool {
        if self.get_ref().len() == 0 {
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.get_ref().len()
    }

    fn add(&mut self, e: Self::Problem) {
        if !self.has(&e) {
            self.0.push(e)
        }
    }

    fn has(&self, c_e: &Self::Problem) -> bool {
        for error in self.get_ref() {
            if error == c_e {
                return true;
            }
        }
        false
    }
}


impl AddAssign for MissingComponentForPackageList {
    fn add_assign(&mut self, rhs: Self) {
        for rhs_e in rhs.get() {
            if !self.has(&rhs_e) {
                self.add(rhs_e)
            }
        }
    }
}

impl AddAssign for RenamedPackageInComponentList {
    fn add_assign(&mut self, rhs: Self) {
        for rhs_e in rhs.get() {
            if !self.has(&rhs_e) {
                self.add(rhs_e)
            }
        }
    }
}

impl AddAssign for UnRunnableMakeCommandList {
    fn add_assign(&mut self, rhs: Self) {
        for rhs_e in rhs.get() {
            if !self.has(&rhs_e) {
                self.add(rhs_e)
            }
        }
    }
}

impl AddAssign for NonExistingRequiredPackageList {
    fn add_assign(&mut self, rhs: Self) {
        for rhs_e in rhs.get() {
            if !self.has(&rhs_e) {
                self.add(rhs_e)
            }
        }
    }
}

impl AddAssign for ObsoletedRequiredPackageList {
    fn add_assign(&mut self, rhs: Self) {
        for rhs_e in rhs.get() {
            if !self.has(&rhs_e) {
                self.add(rhs_e)
            }
        }
    }
}

impl AddAssign for PartlyObsoletedRequiredPackageList {
    fn add_assign(&mut self, rhs: Self) {
        for rhs_e in rhs.get() {
            if !self.has(&rhs_e) {
                self.add(rhs_e)
            }
        }
    }
}

impl AddAssign for UselessComponentsList {
    fn add_assign(&mut self, rhs: Self) {
        for rhs_e in rhs.get() {
            if !self.has(&rhs_e) {
                self.add(rhs_e)
            }
        }
    }
}

pub trait ProblemList {
    type Problem;

    fn get(self) -> Vec<Self::Problem>;

    fn get_ref(&self) -> &Vec<Self::Problem>;

    fn get_ref_mut(&mut self) -> &mut Vec<Self::Problem>;

    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn add(&mut self, e: Self::Problem);

    fn has(&self, c_e: &Self::Problem) -> bool;
}

pub fn report(
    missing_component_for_package_list: MissingComponentForPackageList,
    renamed_needs_renamed_list: RenamedNeedsRenamedList,
    renamed_package_in_component_list: RenamedPackageInComponentList,
    un_runnable_make_command_list: UnRunnableMakeCommandList,
    mut non_existing_required_package_list: NonExistingRequiredPackageList,
    mut obsolete_required_package_list: ObsoletedRequiredPackageList,
    mut partly_obsolete_required_package_list: PartlyObsoletedRequiredPackageList,
    useless_components_list: UselessComponentsList,
    components: &Components,
) {
    let mut partly = 0;
    let mut non_existing = 0;
    let mut renamed = 0;
    let mut obsoleted = 0;

    if !useless_components_list.is_empty() {
        useless_components_list.report()
    }

    if !partly_obsolete_required_package_list.is_empty() {
        partly = partly_obsolete_required_package_list.report(components)
    }

    if !missing_component_for_package_list.is_empty() {
        missing_component_for_package_list.report()
    }

    if !non_existing_required_package_list.is_empty() {
        non_existing = non_existing_required_package_list.report(components)
    }

    if !renamed_needs_renamed_list.is_empty() {
        renamed_needs_renamed_list.report()
    }

    if !renamed_package_in_component_list.is_empty() {
        renamed = renamed_package_in_component_list.report()
    };

    if !obsolete_required_package_list.is_empty() {
        obsoleted = obsolete_required_package_list.report(components)
    }

    if !un_runnable_make_command_list.is_empty() {
        un_runnable_make_command_list.report()
    }

    if !useless_components_list.is_empty() {
        info!("Number of components that are not needed by any package: {}", useless_components_list.len());
    }

    if !partly_obsolete_required_package_list.is_empty() {
        warn!("Number of obsoleted packages with older normal version which are needed as dependency: {}", partly);
    }

    if !missing_component_for_package_list.is_empty() {
        warn!("Number of packages that do not belong to a component: {}", missing_component_for_package_list.len());
    }

    if !non_existing_required_package_list.is_empty() {
        warn!("Number of non existing packages which are needed as dependency: {}", non_existing);
    }

    if !renamed_needs_renamed_list.is_empty() {
        error!("Number of renamed packages that need renamed packages: {}", renamed_needs_renamed_list.len())
    }

    if !renamed_package_in_component_list.is_empty() {
        error!("Number of obsoleted or renamed packages which are in component: {}", renamed);
    }

    if !obsolete_required_package_list.is_empty() {
        error!("Number of obsoleted packages which are needed as dependency: {}", obsoleted);
    }

    if !un_runnable_make_command_list.is_empty() {
        error!("Number of un-runnable make commands: {}", un_runnable_make_command_list.len());
    }
}
