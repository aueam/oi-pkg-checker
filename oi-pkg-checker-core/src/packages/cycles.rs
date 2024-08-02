use std::cmp::Ordering;

use fmri::FMRI;
use log::warn;

use crate::{
    get,
    packages::{
        components::Components, cycles::EdgeType::*, depend_types::DependTypes, package::Package,
    },
    weak_type,
};

#[derive(Clone, PartialEq, Debug, Ord, Eq, PartialOrd)]
pub enum EdgeType {
    RuntimeRequire,
    RuntimeRequireAny,
    RuntimeConditionalFmri,
    Build,
    Test,
    SystemBuild,
    SystemTest,
    New,
}

#[derive(Clone, Debug, Eq)]
pub enum Or {
    Component(String),
    Package(FMRI),
}

impl Components {
    pub fn check_cycles(&mut self, edge_types: &[EdgeType]) -> Vec<Vec<(Or, EdgeType)>> {
        let mut cycles: Vec<Vec<(FMRI, EdgeType)>> = Vec::new();
        let mut visited: Vec<FMRI> = Vec::new();

        let a = self.packages.len();

        for (i, p) in self.packages.iter().enumerate() {
            let package = get!(p).clone();

            println!("{}/{}", i, a);

            if visited.contains(package.get_fmri()) {
                continue;
            }

            find_cycles(
                self,
                package,
                Vec::new(),
                &mut visited,
                &mut cycles,
                edge_types,
                New,
            );
        }

        let mut new_cycles: Vec<Vec<(Or, EdgeType)>> = Vec::new();
        for cycle in cycles {
            let mut new_cycle: Vec<(Or, EdgeType)> = Vec::new();
            let mut last_or = Or::Component(String::from("new"));
            for (f, t) in cycle {
                new_cycle.push((last_or, t));
                last_or =
                    if let Some(c) = get!(self.get_package_by_fmri(&f).unwrap()).is_in_component() {
                        Or::Component(get!(c).get_name().clone())
                    } else {
                        Or::Package(f)
                    }
            }

            new_cycle.first_mut().unwrap().0 = last_or;
            new_cycles.push(new_cycle);
        }

        new_cycles.sort();
        new_cycles.dedup();

        let mut new_new_cycles: Vec<Vec<(Or, EdgeType)>> = Vec::new();

        for cycle in new_cycles {
            let mut new_cycle: Vec<(Or, EdgeType)> = Vec::new();

            for (i, (or, t)) in cycle.iter().enumerate() {
                match t {
                    RuntimeRequire | RuntimeRequireAny | RuntimeConditionalFmri => {
                        if &match cycle.get(i + 1) {
                            None => cycle.first().unwrap(),
                            Some(a) => a,
                        }
                        .0 == or
                        {
                            continue;
                        }
                    }
                    _ => {}
                }

                new_cycle.push((or.clone(), t.clone()));
            }

            new_new_cycles.push(new_cycle);
        }

        new_new_cycles
    }
}

fn find_cycles(
    components: &Components,
    package: Package,
    mut route: Vec<(FMRI, EdgeType)>,
    visited: &mut Vec<FMRI>,
    cycles: &mut Vec<Vec<(FMRI, EdgeType)>>,
    follow_edge_types: &[EdgeType],
    from_type: EdgeType,
) -> bool {
    let current_fmri = package.get_fmri();

    if route_contains(&route, current_fmri) {
        finish_cycle(components, cycles, &route, current_fmri, from_type);
        return true;
    } else {
        route.push((current_fmri.clone(), from_type));
    }

    if visited.contains(current_fmri) {
        return false;
    } else {
        visited.push(current_fmri.clone())
    }

    let mut find = |package: Package, e_type: EdgeType| {
        find_cycles(
            components,
            package,
            route.clone(),
            visited,
            cycles,
            follow_edge_types,
            e_type,
        );
    };

    for r_d in package
        .get_versions()
        .first()
        .unwrap()
        .get_runtime_dependencies()
    {
        let (f, t) = match r_d {
            DependTypes::Require(f) => {
                if !follow_edge_types.contains(&RuntimeRequire) {
                    continue;
                }
                (f, RuntimeRequire)
            }
            DependTypes::RequireAny(_) => continue, // TODO: implement this
            DependTypes::Conditional(f, _) => {
                if !follow_edge_types.contains(&RuntimeConditionalFmri) {
                    continue;
                }
                (f, RuntimeConditionalFmri)
            }
            _ => continue,
        };

        match components.get_package_by_fmri(f) {
            Ok(p) => find(get!(p).clone(), t),
            Err(e) => warn!("cycle finder: {e}"),
        }
    }

    let mut check = |deps: &Vec<weak_type!(Package)>, edge_type: EdgeType| {
        if follow_edge_types.contains(&edge_type) {
            for p in deps.iter().map(|p| p.upgrade().unwrap()) {
                find(get!(p).clone(), edge_type.clone());
            }
        }
    };

    if let Some(c) = package.is_in_component() {
        let component = get!(c);

        check(component.get_build_dependencies(), Build);
        check(component.get_sys_build_dependencies(), SystemBuild);
        check(component.get_test_dependencies(), Test);
        check(component.get_sys_test_dependencies(), SystemTest);
    }

    false
}

fn finish_cycle(
    components: &Components,
    cycles: &mut Vec<Vec<(FMRI, EdgeType)>>,
    route: &Vec<(FMRI, EdgeType)>,
    fmri: &FMRI,
    from_type: EdgeType,
) {
    let mut cycle: Vec<(FMRI, EdgeType)> = Vec::new();
    let mut in_cycle = false;
    let mut only_runtime = match from_type {
        RuntimeRequire | RuntimeRequireAny | RuntimeConditionalFmri | New => true,
        Build | Test | SystemBuild | SystemTest => false,
    };

    // let add = || {
    //     cycle.push((fmri.clone(), t.clone()));
    // };

    for (f, t) in route {
        if in_cycle {
            cycle.push((f.clone(), t.clone()));

            match t {
                RuntimeRequire | RuntimeRequireAny | RuntimeConditionalFmri | New => {}
                Build | Test | SystemBuild | SystemTest => only_runtime = false,
            }
        } else if f.package_name_eq(fmri) {
            cycle.push((fmri.clone(), from_type.clone()));
            in_cycle = true;
        }
    }

    if !only_runtime && !is_cycle_in_one_component(components, &cycle) {
        cycles.push(cycle)
    }
}

fn is_cycle_in_one_component(components: &Components, cycle: &Vec<(FMRI, EdgeType)>) -> bool {
    let c = get!(components
        .get_package_by_fmri(&cycle.first().unwrap().0)
        .unwrap());
    let packages = if let Some(c) = c.is_in_component() {
        get!(c)
    } else if cycle.len() == 1 {
        return true;
    } else if cycle.len() > 1 {
        return false;
    } else {
        panic!("bug?");
    }
    .packages
    .iter()
    .map(|p| get!(p.upgrade().unwrap()).get_fmri().clone())
    .collect::<Vec<FMRI>>();

    for (f, _) in cycle {
        if !packages.contains(f) {
            return false;
        }
    }

    true
}

fn route_contains(route: &Vec<(FMRI, EdgeType)>, fmri: &FMRI) -> bool {
    for (f, _) in route {
        if f.package_name_eq(fmri) {
            return true;
        }
    }
    false
}

pub fn format_cycle(cycle: &Vec<(Or, EdgeType)>) -> String {
    let mut string = String::new();

    for (or, t) in cycle {
        string.push_str(&format!(
            "{}   --{}-->   ",
            match or {
                Or::Component(name) => name.clone(),
                Or::Package(f) => "P|".to_owned() + f.get_package_name_as_ref_string(),
            },
            match t {
                RuntimeRequire => "Require",
                RuntimeRequireAny => "RequireAny",
                RuntimeConditionalFmri => "ConditionalFmri",
                Build => "Build",
                Test => "Test",
                SystemBuild => "SystemBuild",
                SystemTest => "SystemTest",
                New => unimplemented!(),
            }
        ));
    }

    string
}

impl PartialOrd for Or {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Or {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Or::Component(_), Or::Package(_)) => Ordering::Greater,
            (Or::Package(_), Or::Component(_)) => Ordering::Less,
            (Or::Component(a), Or::Component(b)) => a.cmp(b),
            (Or::Package(a), Or::Package(b)) => {
                if a.package_name_eq(b) {
                    Ordering::Equal
                } else {
                    a.cmp(b)
                }
            }
        }
    }
}

impl PartialEq for Or {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Or::Component(_), Or::Package(_)) | (Or::Package(_), Or::Component(_)) => false,
            (Or::Component(a), Or::Component(b)) => a == b,
            (Or::Package(a), Or::Package(b)) => a.package_name_eq(b),
        }
    }
}
