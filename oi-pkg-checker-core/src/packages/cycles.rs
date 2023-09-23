// TODO: remake
// use std::fmt::{Display, Formatter};
// use log::{debug, error, info, warn};
// use crate::assets::open_indiana_oi_userland_git::ComponentPackagesList;
// use fmri::FMRI;
// use crate::packages::component::Component;
// use crate::packages::components::Components;
// use crate::packages::depend_types::DependTypes;
// use crate::packages::dependency::Dependency;
// use crate::packages::dependency_type::DependencyTypes;
//
// #[derive(Clone)]
// pub struct Cycles(Vec<CycleRoute>);
//
// impl Cycles {
//     pub fn new() -> Self {
//         Self(Vec::new())
//     }
//
//     pub fn add(&mut self, cycle: CycleRoute) {
//         if !self.contains(&cycle) {
//             for section in cycle.get_ref() {
//                 if section.get_type_ref() == &DependencyTypes::Build || section.get_type_ref() == &DependencyTypes::SystemBuild {
//                     self.0.push(cycle.clone());
//                     return;
//                 }
//             }
//         }
//     }
//
//     pub fn get(self) -> Vec<CycleRoute> {
//         self.0
//     }
//
//     pub fn get_ref(&self) -> &Vec<CycleRoute> {
//         &self.0
//     }
//
//     pub fn contains(&self, checking_cycle: &CycleRoute) -> bool {
//         for cycle in self.get_ref() {
//             if cycle == checking_cycle {
//                 return true;
//             }
//         }
//         false
//     }
// }
//
// #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
// pub struct CycleRoute(Vec<Section>);
//
// #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
// pub struct Section {
//     pub component_name: String,
//     pub dependency_type: DependencyTypes,
// }
//
// impl CycleRoute {
//     pub fn new(route: Vec<Section>) -> Self {
//         Self(route)
//     }
//
//     pub fn new_empty() -> Self {
//         Self(Vec::new())
//     }
//
//     pub fn add(&mut self, section: Section) {
//         self.0.push(section)
//     }
//
//     pub fn get_ref(&self) -> &Vec<Section> {
//         &self.0
//     }
//
//     pub fn get_ref_mut(&mut self) -> &mut Vec<Section> {
//         &mut self.0
//     }
// }
//
// impl Section {
//     pub fn new(component_name: String, dependency_type: DependencyTypes) -> Self {
//         Self {
//             component_name,
//             dependency_type,
//         }
//     }
//
//     pub fn get_name_ref(&self) -> &String {
//         &self.component_name
//     }
//
//     pub fn get_type_ref(&self) -> &DependencyTypes {
//         &self.dependency_type
//     }
// }
//
//
// impl Component {
//     pub fn find_cycles(
//         &self,
//         cycles: &mut Cycles,
//         was_there: &mut Vec<String>,
//         route: CycleRoute,
//         components: &Components,
//         dependency_types: Vec<DependencyTypes>,
//     ) {
//         let here = self.clone().get_name();
//         for component_or_package in was_there.clone() {
//             if component_or_package == here {
//                 return;
//             }
//         }
//         was_there.push(here.clone());
//
//         for (mut dependency, dependency_type) in self.get_dependencies(dependency_types.clone()) {
//             let mut potential_cycle = route.clone();
//             potential_cycle.add(Section::new(here.clone(), dependency_type));
//             match dependency.get_ref_mut() {
//                 DependTypes::Require(fmri) => {
//                     let component = match components.get_component_name_by_package(fmri) {
//                         Some(component_name) => {
//                             let component = components.get_component_by_name(&component_name);
//                             for (dependency, dependency_type) in component.get_dependencies(dependency_types.clone()) {
//                                 let component_name = match get_component_name_from_dependency(components, dependency) {
//                                     None => continue,
//                                     Some(component_name) => component_name
//                                 };
//
//                                 for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
//                                     if section.get_name_ref() == &component_name {
//                                         let mut potential_cycle = potential_cycle.clone();
//                                         potential_cycle.add(Section::new(component_name.clone(), dependency_type.clone()));
//                                         let cycle = potential_cycle.get_ref_mut().split_off(index);
//                                         debug!("new cycle: {:?}", cycle);
//                                         cycles.add(CycleRoute::new(cycle));
//                                     }
//                                 }
//                             }
//                             component
//                         }
//                         None => continue,
//                     };
//
//                     component.find_cycles(
//                         cycles,
//                         was_there,
//                         potential_cycle.clone(),
//                         components,
//                         dependency_types.clone(),
//                     )
//                 }
//                 DependTypes::Optional(fmri) => {
//                     let component = match components.get_component_name_by_package(fmri) {
//                         Some(component_name) => {
//                             let component = components.get_component_by_name(&component_name);
//                             for (dependency, dependency_type) in component.get_dependencies(dependency_types.clone()) {
//                                 let component_name = match get_component_name_from_dependency(components, dependency) {
//                                     None => continue,
//                                     Some(component_name) => component_name
//                                 };
//
//                                 for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
//                                     if section.get_name_ref() == &component_name {
//                                         let mut potential_cycle = potential_cycle.clone();
//                                         potential_cycle.add(Section::new(component_name.clone(), dependency_type.clone()));
//                                         let cycle = potential_cycle.get_ref_mut().split_off(index);
//                                         debug!("new cycle: {:?}", cycle);
//                                         cycles.add(CycleRoute::new(cycle));
//                                     }
//                                 }
//                             }
//                             component
//                         }
//                         None => continue,
//                     };
//
//                     component.find_cycles(
//                         cycles,
//                         was_there,
//                         potential_cycle.clone(),
//                         components,
//                         dependency_types.clone(),
//                     )
//                 }
//                 DependTypes::Incorporate(fmri) => {
//                     let component = match components.get_component_name_by_package(fmri) {
//                         Some(component_name) => {
//                             let component = components.get_component_by_name(&component_name);
//                             for (dependency, dependency_type) in component.get_dependencies(dependency_types.clone()) {
//                                 let component_name = match get_component_name_from_dependency(components, dependency) {
//                                     None => continue,
//                                     Some(component_name) => component_name
//                                 };
//
//                                 for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
//                                     if section.get_name_ref() == &component_name {
//                                         let mut potential_cycle = potential_cycle.clone();
//                                         potential_cycle.add(Section::new(component_name.clone(), dependency_type.clone()));
//                                         let cycle = potential_cycle.get_ref_mut().split_off(index);
//                                         debug!("new cycle: {:?}", cycle);
//                                         cycles.add(CycleRoute::new(cycle));
//                                     }
//                                 }
//                             }
//                             component
//                         }
//                         None => continue,
//                     };
//
//                     component.find_cycles(
//                         cycles,
//                         was_there,
//                         potential_cycle.clone(),
//                         components,
//                         dependency_types.clone(),
//                     )
//                 }
//                 DependTypes::RequireAny(fmri_list) => {
//                     for fmri in fmri_list.get_ref() {
//                         let component = match components.get_component_name_by_package(fmri) {
//                             Some(component_name) => {
//                                 let component = components.get_component_by_name(&component_name);
//                                 for (dependency, dependency_type) in component.get_dependencies(dependency_types.clone()) {
//                                     let component_name = match get_component_name_from_dependency(components, dependency) {
//                                         None => continue,
//                                         Some(component_name) => component_name
//                                     };
//
//                                     for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
//                                         if section.get_name_ref() == &component_name {
//                                             let mut potential_cycle = potential_cycle.clone();
//                                             potential_cycle.add(Section::new(component_name.clone(), dependency_type.clone()));
//                                             let cycle = potential_cycle.get_ref_mut().split_off(index);
//                                             debug!("new cycle: {:?}", cycle);
//                                             cycles.add(CycleRoute::new(cycle));
//                                         }
//                                     }
//                                 }
//                                 component
//                             }
//                             None => continue,
//                         };
//
//                         component.find_cycles(
//                             cycles,
//                             was_there,
//                             potential_cycle.clone(),
//                             components,
//                             dependency_types.clone(),
//                         )
//                     }
//                 }
//                 DependTypes::Conditional(fmri, predicate) => {
//                     let component = match components.get_component_name_by_package(fmri) {
//                         Some(component_name) => {
//                             let component = components.get_component_by_name(&component_name);
//                             for (dependency, dependency_type) in component.get_dependencies(dependency_types.clone()) {
//                                 let component_name = match get_component_name_from_dependency(components, dependency) {
//                                     None => continue,
//                                     Some(component_name) => component_name
//                                 };
//
//                                 for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
//                                     if section.get_name_ref() == &component_name {
//                                         let mut potential_cycle = potential_cycle.clone();
//                                         potential_cycle.add(Section::new(component_name.clone(), dependency_type.clone()));
//                                         let cycle = potential_cycle.get_ref_mut().split_off(index);
//                                         debug!("new cycle: {:?}", cycle);
//                                         cycles.add(CycleRoute::new(cycle));
//                                     }
//                                 }
//                             }
//                             component
//                         }
//                         None => continue,
//                     };
//
//                     component.find_cycles(
//                         cycles,
//                         was_there,
//                         potential_cycle.clone(),
//                         components,
//                         dependency_types.clone(),
//                     )
//                 }
//                 DependTypes::Group(fmri) => {
//                     let component = match components.get_component_name_by_package(fmri) {
//                         Some(component_name) => {
//                             let component = components.get_component_by_name(&component_name);
//                             for (dependency, dependency_type) in component.get_dependencies(dependency_types.clone()) {
//                                 let component_name = match get_component_name_from_dependency(components, dependency) {
//                                     None => continue,
//                                     Some(component_name) => component_name
//                                 };
//
//                                 for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
//                                     if section.get_name_ref() == &component_name {
//                                         let mut potential_cycle = potential_cycle.clone();
//                                         potential_cycle.add(Section::new(component_name.clone(), dependency_type.clone()));
//                                         let cycle = potential_cycle.get_ref_mut().split_off(index);
//                                         debug!("new cycle: {:?}", cycle);
//                                         cycles.add(CycleRoute::new(cycle));
//                                     }
//                                 }
//                             }
//                             component
//                         }
//                         None => continue,
//                     };
//
//                     component.find_cycles(
//                         cycles,
//                         was_there,
//                         potential_cycle.clone(),
//                         components,
//                         dependency_types.clone(),
//                     )
//                 }
//                 _ => unimplemented!()
//             }
//         }
//     }
// }
//
// // todo: remove this
// pub fn get_component_name_from_dependency(components: &Components, dependency: Dependency) -> Option<String> {
//     match dependency.get_ref() {
//         DependTypes::Require(fmri) => {
//             return match components.get_component_name_by_package(fmri) {
//                 None => {
//                     None
//                 }
//                 Some(component_name) => {
//                     Some(component_name.clone())
//                 }
//             };
//         }
//         DependTypes::Optional(fmri) => {
//             return match components.get_component_name_by_package(fmri) {
//                 None => {
//                     None
//                 }
//                 Some(component_name) => {
//                     Some(component_name.clone())
//                 }
//             };
//         }
//         DependTypes::Incorporate(fmri) => {
//             return match components.get_component_name_by_package(fmri) {
//                 None => {
//                     None
//                 }
//                 Some(component_name) => {
//                     Some(component_name.clone())
//                 }
//             };
//         }
//         DependTypes::RequireAny(fmri_list) => {
//             for fmri in fmri_list.get_ref() {
//                 // TODO:?
//                 return match components.get_component_name_by_package(fmri) {
//                     None => {
//                         None
//                     }
//                     Some(component_name) => {
//                         Some(component_name.clone())
//                     }
//                 };
//             }
//         }
//         DependTypes::Conditional(fmri, _) => {
//             return match components.get_component_name_by_package(fmri) {
//                 None => {
//                     None
//                 }
//                 Some(component_name) => {
//                     Some(component_name.clone())
//                 }
//             };
//         }
//         DependTypes::Group(fmri) => {
//             return match components.get_component_name_by_package(fmri) {
//                 None => {
//                     None
//                 }
//                 Some(component_name) => {
//                     Some(component_name.clone())
//                 }
//             };
//         }
//         _ => unimplemented!()
//     }
//
//     None
// }
//
// fn check_cycles_dependencies(
//     components: &Components,
//     fmri: &FMRI,
//     dependency_types: Vec<DependencyTypes>,
//     route: &CycleRoute,
//     cycles: &mut Cycles,
//     index: usize,
// ) {
//     let dependencies = match components.get_component_by_package(fmri) {
//         None => return,
//         Some(component) => {
//             component.get_dependencies(dependency_types)
//         }
//     };
//
//     for (dependency, d_type) in &dependencies {
//         let checking_fmri = match dependency.get_ref() {
//             DependTypes::Require(checking_fmri) => checking_fmri,
//             DependTypes::Optional(checking_fmri) => checking_fmri,
//             DependTypes::Incorporate(checking_fmri) => checking_fmri,
//             DependTypes::RequireAny(fmri_list) => {
//                 for checking_fmri in fmri_list.get_ref() {
//                     check_cycles_route(components, route.get_ref(), cycles, checking_fmri, d_type, index);
//                 }
//                 continue;
//             }
//             DependTypes::Conditional(checking_fmri, _) => checking_fmri,
//             DependTypes::Group(checking_fmri) => checking_fmri,
//             _ => unimplemented!()
//         };
//         check_cycles_route(components, route.get_ref(), cycles, checking_fmri, d_type, index);
//     }
// }
//
// fn check_cycles_route(
//     components: &Components,
//     route: &Vec<Section>,
//     cycles: &mut Cycles,
//     checking_fmri: &FMRI,
//     d_type: &DependencyTypes,
//     index: usize,
// ) {
//     let component_name = match components.get_component_name_by_package(checking_fmri) {
//         None => return,
//         Some(component_name) => component_name
//     };
//
//     let mut potential_cycle = route.clone();
//     potential_cycle.push(Section::new(component_name.clone(), d_type.clone()));
//
//     for section in route.clone() {
//         if section.get_name_ref() == component_name {
//             let cycle = potential_cycle.clone().split_off(index);
//             debug!("new cycle 2: {:?}", cycle);
//             cycles.add(CycleRoute::new(cycle));
//         }
//     }
// }
//
// impl Display for CycleRoute {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let mut string = String::new();
//
//         let size = self.get_ref().len();
//         for (index, section) in self.get_ref().iter().enumerate() {
//             string.push_str(&format!("{}[{}]", section.component_name, section.dependency_type) as &str);
//             if index != size - 1 {
//                 string.push_str(" => ")
//             }
//         }
//
//         write!(f, "{}", string)
//     }
// }
//
//
// // match dependency.get_ref() {
// //     DependTypes::Require(fmri) => {
// //         match components.get_component_name_by_package(fmri) {
// //             None => {}
// //             Some(component_name) => {
// //                 for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
// //                     if section.get_name_ref() == component_name {
// //                         let cycle = potential_cycle.clone().get_ref_mut().split_off(index);
// //                         debug!("new cycle: {:?}", cycle);
// //                         cycles.add(CycleRoute::new(cycle));
// //                     }
// //                 }
// //             }
// //         }
// //     },
// //     _ => unimplemented!()
// // }
//
// // potential_cycle.add(Section::new(component_name.clone(), dependency_type));
//
// // for (index, section) in potential_cycle.clone().get_ref().iter().enumerate() {
// //
// //
// //
// //
// //
// //
// //     let mut potential_cycle = potential_cycle.clone();
// //     potential_cycle.add(Section::new(component_name.clone(), DependencyTypes::None));
// //
// //     if section.get_name_ref() == component_name {
// //         let cycle = potential_cycle.clone().get_ref_mut().split_off(index);
// //         debug!("new cycle: {:?}", cycle);
// //         cycles.add(CycleRoute::new(cycle));
// //         continue;
// //     }
// //
// //     // check_cycles_dependencies(
// //     //     components,
// //     //     fmri,
// //     //     dependency_types.clone(),
// //     //     &mut potential_cycle.clone(),
// //     //     cycles,
// //     //     index
// //     // );
// // }
