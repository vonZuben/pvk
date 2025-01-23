use krs_quote::krs_quote_with;

use crate::utils::{StrAsCode, VecMap, VkTyName};

use std::cell::{Ref, RefCell};
use std::ops::{Deref, DerefMut};

// a collection of extensions
#[derive(Default)]
pub(crate) struct ExtensionCollection {
    extensions: VecMap<ExtensionName, ExtensionInfo>,
}

impl ExtensionCollection {
    fn dependency_kind(&self, name: VkTyName) -> DependencyKind {
        self.extensions
            .get(ExtensionName::Base { name })
            .map(|i| match i.kind {
                ExtensionKind::Instance => DependencyKind::InstanceExtension,
                ExtensionKind::Device => DependencyKind::DeviceExtension,
            })
            .unwrap_or(DependencyKind::Version)
    }

    pub fn extension_names_iter(&self) -> impl Iterator<Item = &str> + Clone {
        self.extensions
            .iter()
            .map(|e| e.extension_name.name_as_str())
    }

    pub fn extensions(&self) -> impl Iterator<Item = VkTyName> + Clone + use<'_> {
        self.extensions.iter().map(|e| e.extension_name.name())
    }
}

impl Deref for ExtensionCollection {
    type Target = VecMap<ExtensionName, ExtensionInfo>;

    fn deref(&self) -> &Self::Target {
        &self.extensions
    }
}

impl DerefMut for ExtensionCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.extensions
    }
}

fn instance_filter(info: &&ExtensionInfo) -> bool {
    info.instance_command_names.len() > 0 || matches!(info.kind, ExtensionKind::Instance)
}

fn device_filter(info: &&ExtensionInfo) -> bool {
    info.device_command_names.len() > 0 || matches!(info.kind, ExtensionKind::Device)
}

impl krs_quote::ToTokens for ExtensionCollection {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let extensions = self.extensions.iter();

        // structs
        let instance_command_structs =
            extensions
                .clone()
                .filter(instance_filter)
                .map(|e| ExtensionCommandStruct {
                    name: e.extension_name,
                    commands: &e.instance_command_names,
                });
        let device_command_structs =
            extensions
                .clone()
                .filter(device_filter)
                .map(|e| ExtensionCommandStruct {
                    name: e.extension_name,
                    commands: &e.device_command_names,
                });

        // traits
        let instance_command_traits =
            extensions
                .clone()
                .filter(instance_filter)
                .map(|e| ExtensionCommandTrait {
                    name: e.extension_name,
                    commands: &e.instance_command_names,
                });
        let device_command_traits =
            extensions
                .clone()
                .filter(device_filter)
                .map(|e| ExtensionCommandTrait {
                    name: e.extension_name,
                    commands: &e.device_command_names,
                });

        // commands macro
        let instance_command_macros =
            extensions
                .clone()
                .filter(instance_filter)
                .map(|e| ExtensionCommandMacros {
                    name: e.extension_name,
                    mod_name: "instance",
                    commands: &e.instance_command_names,
                });
        let device_command_macros =
            extensions
                .clone()
                .filter(device_filter)
                .map(|e| ExtensionCommandMacros {
                    name: e.extension_name,
                    mod_name: "device",
                    commands: &e.device_command_names,
                });

        // dependency macros
        let instance_dep_macros =
            extensions
                .clone()
                .filter(instance_filter)
                .map(|e| ExtensionLoadsMacros {
                    info: e,
                    suffix: "instance_loads",
                    for_kind: ExtensionKind::Instance,
                });
        let device_dep_macros =
            extensions
                .clone()
                .filter(device_filter)
                .map(|e| ExtensionLoadsMacros {
                    info: e,
                    suffix: "device_loads",
                    for_kind: ExtensionKind::Device,
                });

        let macro_dependency_traits = extensions.clone().map(|e| MacroDependencyTraits {
            info: e,
            all_extensions: self,
        });

        krs_quote_with!(tokens <-

            #[doc(hidden)]
            pub mod extension {
                pub mod instance {
                    pub mod command_traits {
                        use crate::has_command::*;
                        use crate::CommandProvider;
                        {@* {@instance_command_traits}}
                    }
                    #[doc(hidden)]
                    pub mod command_macros {
                        {@* {@instance_command_macros}}
                    }
                    #[doc(hidden)]
                    pub mod command_structs {
                        use super::super::super::*;
                        {@* {@instance_command_structs}}
                    }
                }

                pub mod device {
                    pub mod command_traits {
                        use crate::has_command::*;
                        use crate::CommandProvider;
                        {@* {@device_command_traits}}
                    }
                    #[doc(hidden)]
                    pub mod command_macros {
                        {@* {@device_command_macros}}
                    }
                    #[doc(hidden)]
                    pub mod command_structs {
                        use super::super::super::*;
                        {@* {@device_command_structs}}
                    }
                }
            }

            #[cfg(not(doc))]
            pub mod macro_dependency_traits {
                {@* {@macro_dependency_traits}}
            }

            #[cfg(not(doc))]
            pub mod macro_loads {
                #[doc(hidden)]
                pub mod instance_loads {
                    {@* {@instance_dep_macros}}
                }

                #[doc(hidden)]
                pub mod device_loads {
                    {@* {@device_dep_macros}}
                }
            }
        )
    }
}

struct ExtensionCommandStruct<'a> {
    name: ExtensionName,
    commands: &'a [VkTyName],
}

impl krs_quote::ToTokens for ExtensionCommandStruct<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let command = self.commands.iter();

        krs_quote_with!(tokens <-
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            pub struct {@name} {
                {@*
                    pub {@command}: {@command},
                }
            }

            impl {@name} {
                #[allow(unused_variables)]
                pub fn load(loader: impl FunctionLoader) -> std::result::Result<Self, CommandLoadError> {
                    Ok(Self {
                        {@* {@command} : {@command}::load(loader)?, }
                    })
                }
            }
        );
    }
}

struct ExtensionCommandTrait<'a> {
    name: ExtensionName,
    commands: &'a [VkTyName],
}

impl krs_quote::ToTokens for ExtensionCommandTrait<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let commands = self.commands.iter();
        krs_quote_with!(tokens <-
            #[allow(non_camel_case_types)]
            pub trait {@name} : CommandProvider {@* + {@commands}} {}
            impl<T> {@name} for T where T: CommandProvider {@* + {@commands}} {}
        );
    }
}

struct ExtensionCommandMacros<'a> {
    name: ExtensionName,
    mod_name: &'a str,
    commands: &'a [VkTyName],
}

impl krs_quote::ToTokens for ExtensionCommandMacros<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let commands = self.commands.iter();
        let macro_name = format!("{}_{}", name.name_as_str(), self.mod_name).as_code();
        krs_quote_with!(tokens <-
            #[doc(hidden)]
            #[macro_export]
            macro_rules! {@macro_name} {
                ( $target:ident ) => {
                    {@* $crate::{@commands}!($target {@name}); }
                }
            }
            pub use {@macro_name} as {@name};
        );
    }
}

struct MacroDependencyTraits<'a> {
    info: &'a ExtensionInfo,
    all_extensions: &'a ExtensionCollection,
}

impl krs_quote::ToTokens for MacroDependencyTraits<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.info.extension_name;

        let options = (0..)
            .into_iter()
            .map(|n| krs_quote::Token::from(format!("O{n}")));
        let options = &options;

        let instance_dependencies = self
            .info
            .dependencies
            .as_ref()
            .and_then(|dep| get_instance_dependency_terms(self.all_extensions, dep))
            .map(|deps| {
                let solutions = DependencyTermSolution::from(deps);
                RefCell::new(solutions)
            });

        let device_dependencies = self
            .info
            .dependencies
            .as_ref()
            .and_then(|dep| get_device_dependency_terms(self.all_extensions, dep))
            .map(|deps| {
                let solutions = DependencyTermSolution::from(deps);
                RefCell::new(solutions)
            });

        let dependencies_to_tokens = |deps| {
            krs_quote::to_tokens_closure!(tokens {
                if let &Some(ref deps) = deps {
                    let solutions = SolutionIterator::new(deps);

                    let message = format!("The dependencies for `{}` are not satisfied", name.name_as_str());
                    let notes: Vec<_> = solutions.clone().map(|s| {
                        s.get_solution_terms().iter().map(|t| t.as_str()).my_intersperse(" + ").collect::<String>()
                    }).collect();

                    let label;
                    if notes.len() == 1 {
                        label = "Enable the below Version/Extensions for this."
                    }
                    else {
                        label = "Enable one of the below sets of Version/Extensions for this."
                    }

                    krs_quote_with!(tokens <-
                        #[allow(unused_imports)]
                        use crate::dependencies::*;

                        #[diagnostic::on_unimplemented(
                            message = {@message},
                            label = {@label},
                            {@,* note = {@notes}}
                        )]
                        pub trait HasDependency<O> {}

                        {@*
                            struct {@options};
                            impl<T> HasDependency<{@options}> for T where T: {@solutions} {}
                        }
                    )
                }
                else {
                    krs_quote_with!(tokens <-
                        pub trait HasDependency<O> {}
                        struct O;
                        impl<T> HasDependency<O> for T {}
                    )
                }
            })
        };

        let instance_dependencies = dependencies_to_tokens(&instance_dependencies);

        let device_dependencies = dependencies_to_tokens(&device_dependencies);

        krs_quote_with!(tokens <-
            #[doc(hidden)]
            #[allow(non_snake_case)]
            pub mod {@name} {
                pub mod instance {
                    {@instance_dependencies}
                }

                pub mod device {
                    {@device_dependencies}
                }
            }
        )
    }
}

struct Intersperse<I: Iterator> {
    iter: std::iter::Peekable<I>,
    separator: I::Item,
    sep_next: bool,
}

trait IntoIntersperse: Iterator + Sized {
    fn my_intersperse(self, separator: Self::Item) -> Intersperse<Self> {
        Intersperse {
            iter: self.peekable(),
            separator,
            sep_next: false,
        }
    }
}

impl<I: Iterator> IntoIntersperse for I {}

impl<I: Iterator> Iterator for Intersperse<I>
where
    I::Item: Copy,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sep_next && self.iter.peek().is_some() {
            self.sep_next = false;
            Some(self.separator)
        } else {
            self.sep_next = true;
            self.iter.next()
        }
    }
}

struct ExtensionLoadsMacros<'a> {
    info: &'a ExtensionInfo,
    suffix: &'a str,
    for_kind: ExtensionKind,
}

impl krs_quote::ToTokens for ExtensionLoadsMacros<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.info.extension_name;
        let loads = match name {
            ExtensionName::Base { ref name } => match (self.for_kind, self.info.kind) {
                (ExtensionKind::Instance, ExtensionKind::Instance) => Some(name.as_str()),
                (ExtensionKind::Device, ExtensionKind::Device) => Some(name.as_str()),
                _ => None,
            },
            ExtensionName::Extra { .. } => None,
        }
        .map(|s| format!("{s}{}{}", "\\", "0")) // building the null character in a way that is only seen as a null character in the generated code
        .into_iter();

        let macro_name = format!("{}_{}", name.name_as_str(), self.suffix).as_code();

        krs_quote_with!(tokens <-
            #[doc(hidden)]
            #[macro_export]
            macro_rules! {@macro_name} {
                ( $list:ident ) => {
                    {@* let $list = R($list, unsafe { $crate::VkStrRaw::new({@loads}.as_ptr().cast()) }); } // this works in conjunction with macro code vk-safe-sys
                }
            }
            pub use {@macro_name} as {@name};
        );
    }
}

// used to represent names of commands that are enabled by an extension and possible extra commands when other features/extensions are available
// base: base extension
// extra: feature or extension that adds more commands
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExtensionName {
    Base { name: VkTyName },
    Extra { name: VkTyName },
}

impl ExtensionName {
    pub fn new<'a>(parts: &crate::vk_parse_visitor::VkParseExtensionParts) -> Self {
        match parts {
            crate::vk_parse_visitor::VkParseExtensionParts::Base(name) => ExtensionName::Base {
                name: (*name).into(),
            },
            crate::vk_parse_visitor::VkParseExtensionParts::Extended(terms) => {
                ExtensionName::Extra {
                    name: terms.name().into(),
                }
            }
        }
    }
    fn name_as_str(&self) -> &str {
        match self {
            ExtensionName::Base { name } => name,
            ExtensionName::Extra { name, .. } => name,
        }
    }
    fn name(&self) -> VkTyName {
        match self {
            ExtensionName::Base { name } => *name,
            ExtensionName::Extra { name, .. } => *name,
        }
    }
}

impl krs_quote::ToTokens for ExtensionName {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name();
        krs_quote_with!(tokens <- {@name})
    }
}

// =================================================================
#[derive(Clone, Copy)]
pub enum ExtensionKind {
    Instance,
    Device,
}

impl krs_quote::ToTokens for ExtensionKind {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        match self {
            Self::Instance => krs_quote_with!(tokens <- InstanceExtension),
            Self::Device => krs_quote_with!(tokens <- DeviceExtension),
        }
    }
}

#[derive(Debug)]
enum DependencyKind {
    Version,
    InstanceExtension,
    DeviceExtension,
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
enum DependencyTermSolution {
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

    fn get_solution_terms(&self) -> Vec<VkTyName> {
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
        vec
    }
}

impl krs_quote::ToTokens for DependencyTermSolution {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        match self {
            DependencyTermSolution::Single(vk_ty_name) => {
                krs_quote_with!(tokens <- {@vk_ty_name} +) // output intended for trait bounds
            }
            DependencyTermSolution::And(vec) => {
                for term in vec {
                    term.to_tokens(tokens);
                }
            }
            DependencyTermSolution::Or(index, vec) => {
                unsafe { vec.get_unchecked(*index) }.to_tokens(tokens)
            }
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
    fn new(terms: &'a RefCell<DependencyTermSolution>) -> Self {
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

fn get_device_dependency_terms(
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

fn get_instance_dependency_terms(
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

/// Command Names for a given extension
/// intended to generate code within a instance/device extension_names module
pub struct ExtensionInfo {
    extension_name: ExtensionName,
    instance_command_names: Vec<VkTyName>,
    device_command_names: Vec<VkTyName>,
    kind: ExtensionKind,
    dependencies: Option<DependencyTerm>,
}

impl ExtensionInfo {
    pub fn new(extension_name: ExtensionName, kind: ExtensionKind) -> Self {
        Self {
            extension_name,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
            kind,
            dependencies: Default::default(),
        }
    }
    pub fn push_instance_command(&mut self, command: VkTyName) {
        self.instance_command_names.push(command);
    }
    pub fn push_device_command(&mut self, command: VkTyName) {
        self.device_command_names.push(command);
    }
    pub fn dependencies<'a>(&mut self, dependencies: impl Into<DependencyTerm>) {
        self.dependencies = Some(dependencies.into())
    }
}
