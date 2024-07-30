use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::{Rc, Weak},
};

use fmri::{FMRI, FMRIList};

use crate::{
    DependTypes,
    problems::{
        Problem,
        Problem::{
            MissingComponentForPackage, NonExistingPackageInPkg5, NonExistingRequired,
            NonExistingRequiredByRenamed, ObsoletedPackageInComponent, ObsoletedRequired,
            ObsoletedRequiredByRenamed, PartlyObsoletedRequired, PartlyObsoletedRequiredByRenamed,
            RenamedNeedsRenamed, RenamedPackageInComponent, UselessComponent,
        },
    }, Problems,
};
use crate::packages::{
    dependency_type::{
        DependencyTypes,
        DependencyTypes::{Build, Runtime, SystemBuild, SystemTest, Test},
    },
    package::Package,
    rev_depend_type::{RevDependType, RevDependType::*},
};

#[derive(Default, Clone, Debug)]
pub struct Components {
    /// components in system
    pub(crate) components: Vec<Rc<RefCell<Component>>>,
    pub(crate) hash_components: HashMap<String, Rc<RefCell<Component>>>,
    /// packages in system
    pub(crate) packages: Vec<Rc<RefCell<Package>>>,
    pub(crate) hash_packages: HashMap<String, Rc<RefCell<Package>>>,
    pub problems: Problems,
}

impl Components {
    pub fn add_package(&mut self, package: Package) {
        let package_name = package.fmri.clone().get_package_name_as_string();

        // if package.obsolete {
        //     self.add_obsolete(fmri.clone())
        // }

        // TODO: set package obsolete?

        let rc_package = Rc::new(RefCell::new(package));

        self.packages.push(Rc::clone(&rc_package));
        self.hash_packages.insert(package_name, rc_package);
    }

    pub fn new_component(
        &mut self,
        component_name: String,
        packages: Vec<FMRI>,
    ) -> Result<(), String> {
        let rc_component = Rc::new(RefCell::new(Component::new(component_name.clone())));

        for fmri in packages {
            let res = match self.get_package_by_fmri(&fmri) {
                Ok(rc_package) => {
                    rc_component
                        .borrow_mut()
                        .add_package(Rc::downgrade(rc_package));

                    rc_package
                        .borrow_mut()
                        .set_component(Rc::clone(&rc_component))
                }
                Err(_) => Some(Box::new(NonExistingPackageInPkg5(
                    fmri,
                    component_name.clone(),
                ))),
            };

            if let Some(p) = res {
                self.problems.add_problem(*p);
            }
        }

        self.components.push(Rc::clone(&rc_component));
        self.hash_components.insert(component_name, rc_component);

        Ok(())
    }

    pub fn get_component_by_name(&self, name: &String) -> Result<&Rc<RefCell<Component>>, String> {
        return match self.hash_components.get(name) {
            None => Err(format!("component {} does not exist", name)),
            Some(component) => Ok(component),
        };
    }

    pub fn get_package_by_fmri(&self, fmri: &FMRI) -> Result<&Rc<RefCell<Package>>, String> {
        return match self
            .hash_packages
            .get(fmri.get_package_name_as_ref_string())
        {
            None => Err(format!("package {} does not exist", fmri)),
            Some(package) => Ok(package),
        };
    }

    pub fn get_components(&self) -> &Vec<Rc<RefCell<Component>>> {
        &self.components
    }

    pub fn get_packages(&self) -> &Vec<Rc<RefCell<Package>>> {
        &self.packages
    }

    /// adds repo dependencies (Build, Test, System Build and System Test) into component
    pub fn add_repo_dependencies(
        &mut self,
        component_name: &String,
        dependencies: Vec<FMRI>,
        dependency_type: &DependencyTypes,
    ) -> Result<(), String> {
        for fmri in dependencies {
            let rc_package = if let Ok(p) = self.get_package_by_fmri(&fmri) {
                p
            } else {
                self.problems.add_problem(NonExistingRequired(
                    DependTypes::Require(fmri),
                    dependency_type.clone(),
                    FMRI::parse_raw("none").unwrap(),
                    component_name.clone(),
                ));

                continue;
            };

            let component = self
                .get_component_by_name(component_name)
                .map_err(|e| format!("failed to get component: {}", e))?;

            let mut component_mut = component.borrow_mut();

            match dependency_type {
                Build => component_mut.build.push(Rc::downgrade(rc_package)),
                Test => component_mut.test.push(Rc::downgrade(rc_package)),
                SystemBuild => component_mut.sys_build.push(Rc::downgrade(rc_package)),
                SystemTest => component_mut.sys_test.push(Rc::downgrade(rc_package)),
                Runtime => {
                    return Err("can not insert runtime dependencies into component".to_owned())
                }
            }

            rc_package
                .borrow_mut()
                .add_dependent(Rc::clone(component), dependency_type)
                .map_err(|e| format!("failed to add dependent: {}", e))?;
        }

        Ok(())
    }

    pub fn set_package_obsolete(&mut self, fmri: FMRI) -> Result<(), String> {
        let mut fmri_clone = fmri.clone();
        let rc_package = self
            .get_package_by_fmri(fmri_clone.remove_version())
            .map_err(|e| format!("failed to get package: {}", e))?;

        match fmri.get_version() {
            None => rc_package.borrow_mut().set_obsolete(true),
            Some(fmri_version) => {
                for version in rc_package.borrow_mut().get_versions_mut() {
                    if version.version == fmri_version {
                        version.set_obsolete(true);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn set_package_renamed(&mut self, fmri: FMRI) -> Result<(), String> {
        let mut fmri_clone = fmri.clone();
        let rc_package = self
            .get_package_by_fmri(fmri_clone.remove_version())
            .map_err(|e| format!("failed to get package: {}", e))?;

        match fmri.get_version() {
            None => rc_package.borrow_mut().set_renamed(true),
            Some(fmri_version) => {
                for version in rc_package.borrow_mut().get_versions_mut() {
                    if version.version == fmri_version {
                        version.set_renamed(true);
                    }
                }
            }
        }

        Ok(())
    }

    // TODO: there might be something wrong here
    pub fn distribute_reverse_runtime_dependencies(&mut self) {
        let mut rev_run_deps: HashMap<FMRI, HashSet<RevDependType>> = HashMap::new();

        let mut add = |fmri: FMRI, rev_depend_type: RevDependType| {
            rev_run_deps
                .entry(fmri)
                .or_default()
                .insert(rev_depend_type);
        };

        for p in &*self.packages {
            let package = p.borrow();
            for version in &package.versions {
                for d in &version.runtime {
                    match d.clone() {
                        DependTypes::Require(f) => add(f, Require(package.fmri.clone())),
                        DependTypes::Optional(f) => add(f, Optional(package.fmri.clone())),
                        DependTypes::Incorporate(f) => add(f, Incorporate(package.fmri.clone())),
                        DependTypes::RequireAny(l) => {
                            for f in l.get() {
                                add(f, Require(package.fmri.clone()))
                            }
                        }
                        DependTypes::Conditional(f, p) => {
                            add(f, ConditionalFmri(package.fmri.clone()));
                            add(p, ConditionalPredicate(package.fmri.clone()));
                        }
                        DependTypes::Group(f) => add(f, Group(package.fmri.clone())),
                        _ => unimplemented!(),
                    };
                }
            }
        }

        for (fmri, hash_rev_deps) in rev_run_deps {
            let mut rev_deps = hash_rev_deps
                .iter()
                .cloned()
                .collect::<Vec<RevDependType>>();

            match self.get_package_by_fmri(&fmri) {
                Ok(package) => package
                    .borrow_mut()
                    .runtime_dependents
                    .append(&mut rev_deps),
                Err(_) => {
                    for rev_dep in rev_deps {
                        let (f, d_type) = match rev_dep {
                            Require(f) => (f, DependTypes::Require(fmri.clone())),
                            Optional(f) => (f, DependTypes::Optional(fmri.clone())),
                            Incorporate(f) => (f, DependTypes::Incorporate(fmri.clone())),
                            RequireAny(f) => (
                                f,
                                DependTypes::RequireAny(FMRIList::from(vec![fmri.clone()])),
                            ),
                            ConditionalFmri(f) => (
                                f,
                                DependTypes::Conditional(
                                    fmri.clone(),
                                    FMRI::parse_raw("none").unwrap(),
                                ),
                            ),
                            ConditionalPredicate(f) => (
                                f,
                                DependTypes::Conditional(
                                    FMRI::parse_raw("none").unwrap(),
                                    fmri.clone(),
                                ),
                            ),
                            Group(f) => (f, DependTypes::Group(fmri.clone())),
                        };

                        self.problems
                            .add_problem(match self.get_package_by_fmri(&f) {
                                Ok(p) => match p.borrow().is_renamed() {
                                    true => NonExistingRequiredByRenamed(d_type, Runtime, f),
                                    false => NonExistingRequired(d_type, Runtime, f, "".to_owned()),
                                },
                                Err(_) => {
                                    panic!("non existing as required by non existing?")
                                }
                            });
                    }
                }
            }
        }
    }

    pub fn remove_old_versions(&mut self) {
        for p in &mut self.packages {
            let mut package = p.borrow_mut();

            package.versions.sort_by(|a, b| b.version.cmp(&a.version));

            let mut new_ver = package.versions.first().unwrap().clone();

            for ver in &package.versions {
                if !ver.is_obsolete() && !ver.is_renamed() {
                    new_ver = ver.clone();
                    break;
                }
            }

            package.change_versions(vec![new_ver]);
        }
    }

    pub fn check_problems(&mut self) -> Result<(), String> {
        // ObsoletedPackageInComponent and RenamedPackageInComponent
        for c in &*self.components {
            let component = c.borrow();
            for p in &component.packages {
                let t = p.upgrade().unwrap();
                let package = t.borrow();
                if package.is_obsolete() {
                    self.problems.add_problem(ObsoletedPackageInComponent(
                        package.fmri.clone(),
                        component.name.clone(),
                    ));
                } else if package.is_renamed() {
                    self.problems.add_problem(RenamedPackageInComponent(
                        package.fmri.clone(),
                        component.name.clone(),
                    ));
                }
            }
        }

        // MissingComponentForPackage
        for p in &*self.packages {
            let package = p.borrow();

            if package.is_in_component().is_none()
                && !package.is_renamed()
                && !package.is_obsolete()
            {
                self.problems
                    .add_problem(MissingComponentForPackage(package.fmri.clone()));
            }
        }

        // UselessComponent
        for c in &*self.components {
            let component = c.borrow();
            if component.packages.iter().all(|p| {
                let tmp = p.upgrade().unwrap();
                let package = tmp.borrow();

                if package.is_obsolete() || package.is_renamed() {
                    return false;
                }

                for dep in &package.runtime_dependents {
                    if let Incorporate(_) = dep {
                    } else {
                        return false;
                    }
                }

                if package.build_dependents.is_empty()
                    && package.test_dependents.is_empty()
                    && package.sys_build_dependents.is_empty()
                    && package.sys_build_dependents.is_empty()
                {
                    return true;
                }

                false
            }) {
                self.problems
                    .add_problem(UselessComponent(component.get_name().clone()));
            }
        }

        // RenamedNeedsRenamed
        for p in &*self.packages {
            let package = p.borrow();

            if !package.is_renamed() {
                continue;
            }

            for rev_dep in &package.runtime_dependents {
                match rev_dep {
                    Require(fmri)
                    | Optional(fmri)
                    | Incorporate(fmri)
                    | RequireAny(fmri)
                    | ConditionalFmri(fmri)
                    | ConditionalPredicate(fmri)
                    | Group(fmri) => {
                        let package_b = self
                            .get_package_by_fmri(fmri)
                            .map_err(|e| format!("failed to get package: {}", e))?;
                        if !package_b.borrow().is_renamed() {
                            continue;
                        }
                        let fmri_b = package_b.borrow().fmri.clone();
                        self.problems
                            .add_problem(RenamedNeedsRenamed(fmri_b, package.fmri.clone()));
                    }
                }
            }

            match &package.component {
                None => {}
                Some(c) => {
                    let component = c.borrow();

                    let mut check_dependencies = |dependencies: &Vec<Weak<RefCell<Package>>>| {
                        for dep in dependencies {
                            let p = dep.upgrade().unwrap();
                            let package_b = p.borrow();
                            if package_b.is_renamed() {
                                self.problems.add_problem(RenamedNeedsRenamed(
                                    package.fmri.clone(),
                                    package_b.fmri.clone(),
                                ));
                            }
                        }
                    };

                    check_dependencies(&component.build);
                    check_dependencies(&component.test);
                    check_dependencies(&component.sys_build);
                    check_dependencies(&component.sys_test);
                }
            }
        }

        // ObsoletedRequired, ObsoletedRequiredByRenamed, PartlyObsoletedRequired, PartlyObsoletedRequiredByRenamed
        for p in &self.packages.clone() {
            let package = p.borrow();

            if !package.is_obsolete() {
                continue;
            }

            if package.versions.first().unwrap().is_obsolete() {
                check_obsoleted_required_packages(
                    self,
                    &package,
                    ObsoletedRequired,
                    ObsoletedRequiredByRenamed,
                );
            } else {
                check_obsoleted_required_packages(
                    self,
                    &package,
                    PartlyObsoletedRequired,
                    PartlyObsoletedRequiredByRenamed,
                );
            }
        }

        Ok(())
    }
}

fn check_obsoleted_required_packages(
    components: &mut Components,
    package: &Package,
    problem_type: fn(DependTypes, DependencyTypes, FMRI, String) -> Problem,
    problem_type_renamed: fn(DependTypes, DependencyTypes, FMRI) -> Problem,
) {
    let mut check = |deps: &Vec<Rc<RefCell<Component>>>, dt: DependencyTypes| {
        for c in deps {
            components.problems.add_problem(problem_type(
                DependTypes::Require(package.fmri.clone()),
                dt.clone(),
                FMRI::parse_raw("none").unwrap(),
                c.borrow().name.clone(),
            ));
        }
    };

    check(&package.build_dependents, Build);
    check(&package.sys_build_dependents, SystemBuild);
    check(&package.test_dependents, Test);
    check(&package.sys_test_dependents, SystemTest);

    for d in &package.runtime_dependents {
        let required_by_fmri = match d {
            Require(fmri)
            | Optional(fmri)
            | RequireAny(fmri)
            | ConditionalFmri(fmri)
            | ConditionalPredicate(fmri)
            | Group(fmri) => fmri.clone(),
            Incorporate(_) => continue,
        };

        let p = components
            .get_package_by_fmri(&required_by_fmri)
            .unwrap()
            .borrow();
        let o = p.is_obsolete();
        let r = p.is_renamed();
        drop(p);

        if o {
            continue;
        } else if r {
            components.problems.add_problem(problem_type_renamed(
                DependTypes::Require(package.fmri.clone()),
                Runtime,
                required_by_fmri,
            ));
        } else {
            components.problems.add_problem(problem_type(
                DependTypes::Require(package.fmri.clone()),
                Runtime,
                required_by_fmri,
                "".to_owned(),
            ));
        }
    }
}

/// Component contains name, list of packages in component and dependencies.
#[derive(Clone, Debug)]
pub struct Component {
    pub(crate) name: String,
    /// contains no version
    pub(crate) packages: Vec<Weak<RefCell<Package>>>,
    /// dependencies
    pub(crate) build: Vec<Weak<RefCell<Package>>>,
    pub(crate) test: Vec<Weak<RefCell<Package>>>,
    pub(crate) sys_build: Vec<Weak<RefCell<Package>>>,
    pub(crate) sys_test: Vec<Weak<RefCell<Package>>>,
}

impl Component {
    pub fn new(component_name: String) -> Self {
        Self {
            name: component_name,
            packages: Vec::new(),
            build: Vec::new(),
            test: Vec::new(),
            sys_build: Vec::new(),
            sys_test: Vec::new(),
        }
    }

    fn add_package(&mut self, package: Weak<RefCell<Package>>) {
        self.packages.push(package)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}
