use krs_quote::krs_quote_with;

use crate::utils::{StrAsCode, VecMap, VkTyName};

use std::fmt::Write;
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

        let extension_names = self.extensions.iter().map(|e| e.extension_name);

        // structs
        let instance_structs =
            extensions
                .clone()
                .filter(instance_filter)
                .map(|e| ExtensionStruct {
                    name: e.extension_name,
                    commands: &e.instance_command_names,
                });
        let device_structs = extensions
            .clone()
            .filter(device_filter)
            .map(|e| ExtensionStruct {
                name: e.extension_name,
                commands: &e.device_command_names,
            });

        // traits
        let instance_traits = extensions
            .clone()
            .filter(instance_filter)
            .map(|e| ExtensionTrait {
                name: e.extension_name,
                commands: &e.instance_command_names,
            });
        let device_traits = extensions
            .clone()
            .filter(device_filter)
            .map(|e| ExtensionTrait {
                name: e.extension_name,
                commands: &e.device_command_names,
            });

        // commands macro
        let instance_macros =
            extensions
                .clone()
                .filter(instance_filter)
                .map(|e| ExtensionCommandMacros {
                    name: e.extension_name,
                    mod_name: "instance",
                    commands: &e.instance_command_names,
                });
        let device_macros =
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
                .map(|e| ExtensionDependencyMacros {
                    info: e,
                    suffix: "instance_loads",
                    for_kind: ExtensionKind::Instance,
                    all_extensions: self,
                });
        let device_dep_macros =
            extensions
                .clone()
                .filter(device_filter)
                .map(|e| ExtensionDependencyMacros {
                    info: e,
                    suffix: "device_loads",
                    for_kind: ExtensionKind::Device,
                    all_extensions: self,
                });

        krs_quote_with!(tokens <-

            #[doc(hidden)]
            pub mod extension {
                pub mod instance {
                    pub mod traits {
                        use crate::has_command::*;
                        use crate::CommandProvider;
                        {@* {@instance_traits}}
                    }
                    #[doc(hidden)]
                    pub mod macros {
                        {@* {@instance_macros}}
                    }
                    #[doc(hidden)]
                    pub mod structs {
                        use super::super::super::*;
                        {@* {@instance_structs}}
                    }
                }

                pub mod device {
                    pub mod traits {
                        use crate::has_command::*;
                        use crate::CommandProvider;
                        {@* {@device_traits}}
                    }
                    #[doc(hidden)]
                    pub mod macros {
                        {@* {@device_macros}}
                    }
                    #[doc(hidden)]
                    pub mod structs {
                        use super::super::super::*;
                        {@* {@device_structs}}
                    }
                }
            }

            #[cfg(not(doc))]
            pub mod dependencies {
                #[doc(hidden)]
                pub mod traits {
                    {@* #[allow(non_camel_case_types)] pub trait {@extension_names} {} }
                }

                #[doc(hidden)]
                pub mod instance {
                    {@* {@instance_dep_macros}}
                }

                #[doc(hidden)]
                pub mod device {
                    {@* {@device_dep_macros}}
                }
            }
        )
    }
}

struct ExtensionStruct<'a> {
    name: ExtensionName,
    commands: &'a [VkTyName],
}

impl krs_quote::ToTokens for ExtensionStruct<'_> {
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

struct ExtensionTrait<'a> {
    name: ExtensionName,
    commands: &'a [VkTyName],
}

impl krs_quote::ToTokens for ExtensionTrait<'_> {
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

struct ExtensionDependencyMacros<'a> {
    info: &'a ExtensionInfo,
    suffix: &'a str,
    for_kind: ExtensionKind,
    all_extensions: &'a ExtensionCollection,
}

impl krs_quote::ToTokens for ExtensionDependencyMacros<'_> {
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

        let instance_dependencies = self.info.dependencies.as_ref().and_then(|dep| {
            get_instance_dependency_terms(self.all_extensions, dep).map(DependencyTermMeta::from)
        });
        let device_dependencies = self.info.dependencies.as_ref().and_then(|dep| {
            get_device_dependency_terms(self.all_extensions, dep).map(DependencyTermMeta::from)
        });

        let (main_dependencies, secondary_dependencies) = match self.for_kind {
            ExtensionKind::Instance => (instance_dependencies, None),
            ExtensionKind::Device => (device_dependencies, instance_dependencies),
        };

        let mod_name = match self.for_kind {
            ExtensionKind::Instance => "instance".as_code(),
            ExtensionKind::Device => "device".as_code(),
        };

        fn dependency_parts(
            dep: Option<&DependencyTermMeta>,
        ) -> (
            Option<DependencyTermTraits>,
            impl Iterator<Item = VkTyName> + Clone,
            Option<crate::utils::TokenWrapper>,
        ) {
            (
                dep.map(DependencyTermTraits::from),
                dep.map(|dep| dep.name).into_iter(),
                dep.map(|dep| {
                    String::from_iter(
                        (0..dep.number_of_options)
                            .into_iter()
                            .map(|n| format!("O{n},")),
                    )
                    .as_code()
                }),
            )
        }

        let (main_dependency_traits, main_dep_name, main_options) =
            dependency_parts(main_dependencies.as_ref());

        let (secondary_dependency_traits, secondary_dep_name, secondary_options) =
            dependency_parts(secondary_dependencies.as_ref());

        let secondary_deps = match self.for_kind {
            ExtensionKind::Instance => None,
            ExtensionKind::Device => Some(krs_quote::ToTokensClosure(
                |tokens: &mut krs_quote::TokenStream| {
                    krs_quote_with!(tokens <-
                        #[doc(hidden)]
                        pub mod instance {
                            #[allow(unused_imports)]
                            use crate::extension::instance::traits::*;
                            #[allow(unused_imports)]
                            use crate::version::instance::traits::*;

                            #[doc(hidden)]
                            pub trait HasDependency<O> {}
                            impl<I, {@secondary_options}> HasDependency<({@secondary_options})> for I {@* where I: {@secondary_dep_name}<{@secondary_options}> } {}

                            {@secondary_dependency_traits}
                        }
                    )
                },
            )),
        };

        let macro_name = format!("{}_{}", name.name_as_str(), self.suffix).as_code();

        krs_quote_with!(tokens <-

            #[doc(hidden)]
            #[allow(non_snake_case)]
            pub mod {@name} {
                #[allow(unused_imports)]
                use crate::dependencies::traits::*;
                #[allow(unused_imports)]
                use crate::version::{@mod_name}::traits::*;

                #[doc(hidden)]
                pub const fn check_dependencies<T {@* : {@main_dep_name}<{@main_options}>, {@main_options} }>
                    (_infer: std::marker::PhantomData<T>) {}

                {@main_dependency_traits}

                {@secondary_deps}
            }

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

struct DependencyTermTraits<'a> {
    dependencies: &'a DependencyTermMeta,
}

impl krs_quote::ToTokens for DependencyTermTraits<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        match self.dependencies.inner {
            DependencyTermMetaInner::Single(_) => {}
            DependencyTermMetaInner::And(ref terms) => {
                let name = self.dependencies.name;
                let mut option_count = 0;

                let sub_terms: Vec<_> = terms
                    .iter()
                    .map(|term| {
                        let mut options = String::with_capacity(4 * term.number_of_options); // assuming single digit number of options, len of "O#, " is 4
                        for n in std::iter::from_fn(|| {
                            let tmp = option_count;
                            option_count += 1;
                            Some(tmp)
                        })
                        .take(term.number_of_options)
                        {
                            write!(options, "O{n}, ").unwrap()
                        }
                        format!("{}<{}>", term.name, options).as_code()
                    })
                    .collect();

                let options = (0..self.dependencies.number_of_options)
                    .into_iter()
                    .map(|n| format!("O{n}").as_code());

                krs_quote_with!(tokens <-
                    #[doc(hidden)]
                    #[allow(non_camel_case_types)]
                    pub trait {@name}<{@,* {@options}}> {}
                    impl<T, {@,* {@options}}> {@name}<{@,* {@options}}> for T where T: {@+* {@sub_terms}} {}
                );

                for term in terms.iter() {
                    DependencyTermTraits { dependencies: term }.to_tokens(tokens);
                }
            }
            DependencyTermMetaInner::Or(ref terms) => {
                let name = self.dependencies.name;
                let mut option_count = 1;

                let sub_terms: Vec<_> = terms
                    .iter()
                    .map(|term| {
                        let mut options = String::with_capacity(4 * term.number_of_options); // assuming single digit number of options, len of "O#, " is 4
                        for n in std::iter::from_fn(|| {
                            let tmp = option_count;
                            option_count += 1;
                            Some(tmp)
                        })
                        .take(term.number_of_options)
                        {
                            write!(options, "O{n}, ").unwrap()
                        }
                        format!("{}<{}>", term.name, options).as_code()
                    })
                    .collect();

                let sub_options = (1..self.dependencies.number_of_options)
                    .into_iter()
                    .map(|n| format!("O{n}").as_code());

                let option_names: Vec<_> = terms
                    .iter()
                    .map(|term| format!("O_{}", term.name).as_code())
                    .collect();

                krs_quote_with!(tokens <-
                    #[doc(hidden)]
                    #[allow(non_camel_case_types)]
                    pub trait {@name}<O0, {@,* {@sub_options}}> {}
                    {@*
                        #[doc(hidden)]
                        #[allow(non_camel_case_types)]
                        pub struct {@option_names};
                    }

                    {@* impl<T, {@,* {@sub_options}}> {@name}<{@option_names}, {@,* {@sub_options}}> for T where T: {@sub_terms} {} }
                );

                for term in terms.iter() {
                    DependencyTermTraits { dependencies: term }.to_tokens(tokens);
                }
            }
        }
    }
}

impl<'a> From<&'a DependencyTermMeta> for DependencyTermTraits<'a> {
    fn from(dependencies: &'a DependencyTermMeta) -> Self {
        Self { dependencies }
    }
}

#[derive(Debug)]
pub enum DependencyKind {
    Version,
    InstanceExtension,
    DeviceExtension,
}

#[derive(Clone)]
struct DependencyTermMeta {
    name: VkTyName,
    number_of_options: usize,
    inner: DependencyTermMetaInner,
}

impl std::fmt::Debug for DependencyTermMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependencyTerm")
            .field("name", &self.name.as_str())
            .field("number_of_options", &self.number_of_options)
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Clone)]
enum DependencyTermMetaInner {
    Single(VkTyName),
    And(Vec<DependencyTermMeta>),
    Or(Vec<DependencyTermMeta>),
}

impl DependencyTermMetaInner {
    fn name(&self) -> VkTyName {
        match self {
            DependencyTermMetaInner::Single(name) => *name,
            DependencyTermMetaInner::And(ref terms) => {
                let mut terms = terms.iter().peekable();
                let mut name = String::new();
                while let Some(term) = terms.next() {
                    if terms.peek().is_some() {
                        name.push_str(term.name.as_str());
                        name.push_str("__AND__");
                    } else {
                        name.push_str(term.name.as_str());
                    }
                }
                name.into()
            }
            DependencyTermMetaInner::Or(ref terms) => {
                let mut terms = terms.iter().peekable();
                let mut name = String::new();
                while let Some(term) = terms.next() {
                    if terms.peek().is_some() {
                        name.push_str(term.name.as_str());
                        name.push_str("__OR__");
                    } else {
                        name.push_str(term.name.as_str());
                    }
                }
                name.into()
            }
        }
    }

    fn number_of_options(&self) -> usize {
        match self {
            DependencyTermMetaInner::Single(_) => 0,
            DependencyTermMetaInner::And(ref terms) => terms
                .iter()
                .fold(0, |count, term| count + term.number_of_options),
            DependencyTermMetaInner::Or(ref terms) => terms
                .iter()
                .fold(1, |count, term| count + term.number_of_options),
        }
    }
}

impl std::fmt::Debug for DependencyTermMetaInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(arg0) => f.debug_tuple("Single").field(&arg0.as_str()).finish(),
            Self::And(arg0) => f.debug_tuple("And").field(arg0).finish(),
            Self::Or(arg0) => f.debug_tuple("Or").field(arg0).finish(),
        }
    }
}

impl From<DependencyTerm> for DependencyTermMeta {
    fn from(raw_term: DependencyTerm) -> Self {
        let inner: DependencyTermMetaInner = raw_term.into();
        Self {
            name: inner.name(),
            number_of_options: inner.number_of_options(),
            inner,
        }
    }
}

impl From<DependencyTerm> for DependencyTermMetaInner {
    fn from(raw_term: DependencyTerm) -> Self {
        match raw_term {
            DependencyTerm::Single(name) => Self::Single(name),
            DependencyTerm::And(terms) => {
                Self::And(terms.into_iter().map(|rt| rt.into()).collect())
            }
            DependencyTerm::Or(terms) => Self::Or(terms.into_iter().map(|rt| rt.into()).collect()),
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
