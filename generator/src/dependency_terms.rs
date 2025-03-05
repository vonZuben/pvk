use std::cell::{Ref, RefCell};

use crate::extensions::ExtensionCollection;
use crate::utils::VkTyName;

#[derive(Debug)]
pub enum DependencyKind {
    Version,
    InstanceExtension,
    DeviceExtension,
}

#[derive(Clone)]
pub enum DependencyTerm {
    Single(VkTyName),
    And(Vec<DependencyTerm>),
    Or(Vec<DependencyTerm>),
}

impl DependencyTerm {
    fn simplify(&mut self) {
        match self {
            Self::Single(_) => {}
            Self::And(ref mut deps) => {
                for dep in deps.iter_mut() {
                    dep.simplify()
                }
                if deps.len() == 1 {
                    *self = deps.pop().unwrap()
                }
            }
            Self::Or(ref mut deps) => {
                for dep in deps.iter_mut() {
                    dep.simplify()
                }
                if deps.len() == 1 {
                    panic!("OR with len 1 during simplify")
                }
            }
        }
    }
}

impl std::fmt::Debug for DependencyTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(arg0) => f.debug_tuple("Single").field(&arg0.as_str()).finish(),
            Self::And(arg0) => f.debug_tuple("And").field(arg0).finish(),
            Self::Or(arg0) => f.debug_tuple("Or").field(arg0).finish(),
        }
    }
}

impl From<&[VkTyName]> for DependencyTerm {
    fn from(value: &[VkTyName]) -> Self {
        if value.len() == 1 {
            Self::Single(value[0])
        } else {
            Self::And(value.iter().map(|&v| Self::Single(v)).collect())
        }
    }
}

impl From<&crate::vk_parse_visitor::Term<'_>> for DependencyTerm {
    fn from(value: &crate::vk_parse_visitor::Term<'_>) -> Self {
        match value {
            crate::vk_parse_visitor::Term::Single(s) => Self::Single(VkTyName::from(*s)),
            crate::vk_parse_visitor::Term::And(terms) => {
                Self::And(terms.into_iter().map(|term| term.into()).collect())
            }
            crate::vk_parse_visitor::Term::Or(terms) => {
                Self::Or(terms.into_iter().map(|term| term.into()).collect())
            }
        }
    }
}

impl From<crate::vk_parse_visitor::Term<'_>> for DependencyTerm {
    fn from(value: crate::vk_parse_visitor::Term<'_>) -> Self {
        Self::from(&value) // From<&crate::vk_parse_visitor::Term<'_>>
    }
}

#[derive(Clone, Copy)]
enum TermExtensionKind {
    Mix,
    Instance,
    Device,
    Unknown,
}

impl TermExtensionKind {
    fn with(self, other: TermExtensionKind) -> Self {
        match self {
            TermExtensionKind::Mix => Self::Mix,
            TermExtensionKind::Instance => match other {
                TermExtensionKind::Mix => Self::Mix,
                TermExtensionKind::Instance => Self::Instance,
                TermExtensionKind::Device => Self::Mix,
                TermExtensionKind::Unknown => Self::Instance,
            },
            TermExtensionKind::Device => match other {
                TermExtensionKind::Mix => Self::Mix,
                TermExtensionKind::Instance => Self::Mix,
                TermExtensionKind::Device => Self::Device,
                TermExtensionKind::Unknown => Self::Device,
            },
            TermExtensionKind::Unknown => match other {
                TermExtensionKind::Mix => Self::Mix,
                TermExtensionKind::Instance => Self::Instance,
                TermExtensionKind::Device => Self::Device,
                TermExtensionKind::Unknown => Self::Unknown,
            },
        }
    }
}

pub fn get_device_dependency_terms(
    extensions: &ExtensionCollection,
    dependencies: &DependencyTerm,
) -> Option<DependencyTerm> {
    match get_term_kind(extensions, dependencies) {
        TermExtensionKind::Mix => match dependencies {
            DependencyTerm::Single(_) => panic!("can't have mixed kind single dependency"),
            DependencyTerm::And(ref deps) => Some(DependencyTerm::And(
                deps.iter()
                    .filter_map(|dep| get_device_dependency_terms(extensions, dep))
                    .collect(),
            )),
            DependencyTerm::Or(ref deps) => Some(DependencyTerm::Or(
                deps.iter()
                    .filter_map(|dep| get_device_dependency_terms(extensions, dep))
                    .collect(),
            )),
        },
        TermExtensionKind::Instance => None,
        TermExtensionKind::Device => Some(dependencies.clone()),
        TermExtensionKind::Unknown => match dependencies {
            DependencyTerm::Single(_) => Some(dependencies.clone()), // Version dependencies are Unknown kind, and we assume it is the same as a device dependency
            _ => panic!("dependency kind should not be unknown unless is is Single"),
        },
    }
    .map(|mut dt| {
        dt.simplify();
        dt
    })
}

pub fn get_instance_dependency_terms(
    extensions: &ExtensionCollection,
    dependencies: &DependencyTerm,
) -> Option<DependencyTerm> {
    match get_term_kind(extensions, dependencies) {
        TermExtensionKind::Mix => match dependencies {
            DependencyTerm::Single(_) => panic!("can't have mixed kind single dependency"),
            DependencyTerm::And(ref deps) => Some(DependencyTerm::And(
                deps.iter()
                    .filter_map(|dep| get_instance_dependency_terms(extensions, dep))
                    .collect(),
            )),
            DependencyTerm::Or(ref deps) => Some(DependencyTerm::Or(
                deps.iter()
                    .filter_map(|dep| get_instance_dependency_terms(extensions, dep))
                    .collect(),
            )),
        },
        TermExtensionKind::Instance => Some(dependencies.clone()),
        TermExtensionKind::Device => None,
        TermExtensionKind::Unknown => match dependencies {
            DependencyTerm::Single(_) => Some(dependencies.clone()), // Version dependencies are Unknown kind, and we assume it is the same as a device dependency
            _ => panic!("dependency kind should not be unknown unless is is Single"),
        },
    }
    .map(|mut dt| {
        dt.simplify();
        dt
    })
}

fn get_term_kind(
    extensions: &ExtensionCollection,
    dependencies: &DependencyTerm,
) -> TermExtensionKind {
    match dependencies {
        DependencyTerm::Single(dep) => match extensions.dependency_kind(*dep) {
            DependencyKind::Version => TermExtensionKind::Unknown,
            DependencyKind::InstanceExtension => TermExtensionKind::Instance,
            DependencyKind::DeviceExtension => TermExtensionKind::Device,
        },
        DependencyTerm::And(ref deps) => {
            deps.iter().fold(TermExtensionKind::Unknown, |kind, dep| {
                kind.with(get_term_kind(extensions, dep))
            })
        }
        DependencyTerm::Or(ref deps) => {
            deps.iter().fold(TermExtensionKind::Unknown, |kind, dep| {
                kind.with(get_term_kind(extensions, dep))
            })
        }
    }
}

/// Identify possible solutions of the terms
///
/// Terms that include 'Or' clauses have different permutations
/// of solutions for all combinations of options for all
/// 'Or' clauses.
///
/// This identifies a specific permutation with an index for each 'Or'
/// clause. The first/default solution is an arbitrary solution with
/// zero index for all 'Or' clauses.
///
/// Each permutation can be iterated through by incrementing the
/// index with [`increment`].
pub enum DependencyTermSolution {
    Single(VkTyName),
    And(Vec<DependencyTermSolution>),
    Or(usize, Vec<DependencyTermSolution>),
}

/// Report increment result
enum IncrementResult {
    /// There are no 'Or' clauses and nothing to increment
    NoIncrement,
    /// Has incremented
    Incremented,
    /// Has incremented and looped back to the first/default solution
    LoopBack,
}

impl DependencyTermSolution {
    /// increment to the next possible solution
    ///
    /// Recursively check the Term tree and increment the
    /// index of one `Or` clause at a time. When the index of
    /// and `Or` clause has been incremented enough times to
    /// loop back to the first/default index, increment the
    /// next `Or` claus ein the tree.
    ///
    /// IncrementResult::LoopBack will be reported when
    /// all solutions have been iterated, and it has looped back
    /// to the first/default solution.
    fn increment(&mut self) -> IncrementResult {
        match self {
            DependencyTermSolution::Single(_) => IncrementResult::NoIncrement,
            DependencyTermSolution::And(vec) => {
                use IncrementResult::*;

                let mut result = NoIncrement;
                for term in vec.iter_mut() {
                    match term.increment() {
                        NoIncrement => {}
                        Incremented => {
                            result = Incremented;
                            break;
                        }
                        LoopBack => {
                            result = LoopBack;
                        }
                    }
                }
                result
            }
            DependencyTermSolution::Or(index, vec) => {
                use IncrementResult::*;

                let mut result = NoIncrement;
                for term in vec.iter_mut() {
                    match term.increment() {
                        NoIncrement => {}
                        Incremented => {
                            result = Incremented;
                            break;
                        }
                        LoopBack => {
                            result = LoopBack;
                        }
                    }
                }

                match result {
                    NoIncrement | LoopBack => {
                        *index += 1;
                        if *index == vec.len() {
                            *index = 0;
                            LoopBack
                        } else {
                            Incremented
                        }
                    }
                    Incremented => Incremented,
                }
            }
        }
    }

    /// Provide the terms of the current solution
    ///
    /// All extensions are considered when providing the terms in order
    /// to remove redundant terms (e.g. if a solution requires multiple versions
    /// which are redundant, or extensions which are redundant with included version)
    fn get_solution_terms(
        &self,
        all_extensions: &ExtensionCollection,
    ) -> solution_terms::SolutionTerms {
        fn get_solution_helper(solution: &DependencyTermSolution, terms: &mut Vec<VkTyName>) {
            match solution {
                DependencyTermSolution::Single(vk_ty_name) => terms.push(*vk_ty_name),
                DependencyTermSolution::And(vec) => {
                    for term in vec {
                        get_solution_helper(term, terms);
                    }
                }
                DependencyTermSolution::Or(index, vec) => {
                    get_solution_helper(unsafe { vec.get_unchecked(*index) }, terms)
                }
            };
        }

        let mut vec = Vec::new();
        get_solution_helper(self, &mut vec);
        solution_terms::SolutionTerms::new(vec, all_extensions)
    }
}

pub struct SolutionCollection(Vec<solution_terms::SolutionTerms>);

impl SolutionCollection {
    pub fn new(
        solutions: &RefCell<DependencyTermSolution>,
        all_extensions: &ExtensionCollection,
    ) -> Self {
        let mut solutions: Vec<_> = SolutionIterator::new(solutions)
            .map(|s| s.get_solution_terms(all_extensions))
            .collect();
        // the Solution terms have redundant terms removed, which may result in duplicates
        // so remove the duplicates
        // we assume that duplicates will be next to each other due to how
        // the terms are parsed in the first place, so no sorting is needed
        // solutions.sort();
        solutions.dedup();
        Self(solutions)
    }

    pub fn iter(&self) -> impl Iterator<Item = &solution_terms::SolutionTerms> + Clone {
        self.0.iter()
    }
}

mod solution_terms {
    use crate::utils::VkTyName;

    /// Terms of a particular solution
    ///
    /// When this is created,
    #[derive(PartialEq, Eq)]
    pub struct SolutionTerms(Vec<VkTyName>);

    impl SolutionTerms {
        /// Store the terms of a particular solution
        ///
        /// Removes redundant terms when created.
        ///
        /// The order of the provided terms of a particular solution
        /// must be consistent between with respect to all possible solutions \
        /// to the same extension dependencies. Otherwise, different solutions
        /// may not compare properly after redundancies are removed.
        pub fn new(terms: Vec<VkTyName>, all_extensions: &super::ExtensionCollection) -> Self {
            use crate::features::FeatureVersion;

            #[derive(Clone, Copy)]
            enum Scope {
                Version(FeatureVersion),
                Extension,
            }

            impl Scope {
                /// Check if this scope encompasses another scope
                ///
                /// If the scope is defined by a version, then larger versions encompass smaller versions (for now)
                /// TODO, if Vulkan 2.x is ever released, it should be considered how functionality from 1.x if maintained or depreciated
                ///
                /// If the scopes are defined by Extensions, then assume they are not the same
                fn encompasses(&self, other: &Self) -> bool {
                    match (self, other) {
                        (Scope::Version(lhs), Scope::Version(rhs)) => lhs > rhs,
                        _ => false,
                    }
                }

                fn max_version(self, other: Self) -> Self {
                    match (self, other) {
                        (Scope::Version(lhs), Scope::Version(rhs)) => {
                            Scope::Version(std::cmp::max(lhs, rhs))
                        }
                        (Scope::Version(_), Scope::Extension) => self,
                        (Scope::Extension, Scope::Version(_)) => other,
                        (Scope::Extension, Scope::Extension) => self,
                    }
                }
            }

            /// Get scope of the term, and the possible promoted scope
            fn get_scope(
                term: VkTyName,
                all_extensions: &crate::extensions::ExtensionCollection,
            ) -> (Scope, Scope) {
                let mut scope = Scope::Extension;
                let mut promoted_scope = Scope::Extension;

                match all_extensions.find(term) {
                    // term is an extension, so check if it was promoted to a version
                    Some(ex) => match ex.promoted_to() {
                        Some(promoted) => {
                            if all_extensions.find(promoted).is_none() {
                                // must have been promoted to a version
                                promoted_scope =
                                    Scope::Version(crate::features::parse_version(&promoted));
                            }
                        }
                        None => {}
                    },
                    // term must be a version
                    None => scope = Scope::Version(crate::features::parse_version(&term)),
                }

                (scope, promoted_scope)
            }

            let scopes: Vec<_> = terms
                .iter()
                .map(|term| get_scope(*term, all_extensions))
                .collect();
            let max_version = scopes
                .iter()
                .fold(Scope::Extension, |cur, (scope, _)| cur.max_version(*scope));

            let removed_redundant: Vec<_> = terms
                .into_iter()
                .zip(scopes)
                .filter(|(_, (scope, promoted_scope))| {
                    !max_version.encompasses(&scope.max_version(*promoted_scope))
                })
                .map(|(term, _)| term)
                .collect();

            Self(removed_redundant)
        }
        pub fn iter(&self) -> impl Iterator<Item = &VkTyName> + Clone + use<'_> {
            self.0.iter()
        }
    }
}

#[derive(Clone)]
struct SolutionIterator<'a> {
    /// RefCell is used since we cannot use GAT in Iterator trait
    terms: &'a RefCell<DependencyTermSolution>,
    start: bool,
}

impl<'a> SolutionIterator<'a> {
    pub fn new(terms: &'a RefCell<DependencyTermSolution>) -> Self {
        Self { terms, start: true }
    }
}

impl<'a> Iterator for SolutionIterator<'a> {
    type Item = Ref<'a, DependencyTermSolution>;

    fn next(&mut self) -> Option<Self::Item> {
        use IncrementResult::*;

        if self.start {
            self.start = false;
            Some(self.terms.borrow())
        } else {
            let result = self.terms.borrow_mut().increment();
            match result {
                NoIncrement | LoopBack => {
                    self.start = true;
                    None
                }
                Incremented => Some(self.terms.borrow()),
            }
        }
    }
}

impl From<DependencyTerm> for DependencyTermSolution {
    fn from(value: DependencyTerm) -> Self {
        match value {
            DependencyTerm::Single(vk_ty_name) => Self::Single(vk_ty_name),
            DependencyTerm::And(vec) => {
                Self::And(vec.into_iter().map(DependencyTermSolution::from).collect())
            }
            DependencyTerm::Or(vec) => Self::Or(
                0,
                vec.into_iter().map(DependencyTermSolution::from).collect(),
            ),
        }
    }
}
