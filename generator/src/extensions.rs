use krs_quote::krs_quote_with;

use crate::dependency_terms::{
    get_device_dependency_terms, get_instance_dependency_terms, DependencyKind, DependencyTerm,
    DependencyTermSolution, SolutionCollection,
};
use crate::utils::{IntoIntersperse, StrAsCode, VecMap, VkTyName};

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

// a collection of extensions
#[derive(Default)]
pub(crate) struct ExtensionCollection {
    extensions: VecMap<ExtensionName, ExtensionInfo>,
}

impl ExtensionCollection {
    pub fn dependency_kind(&self, name: VkTyName) -> DependencyKind {
        self.extensions
            .get(ExtensionName::Base { name })
            .map(|i| match i.kind {
                ExtensionKind::Instance => DependencyKind::InstanceExtension,
                ExtensionKind::Device => DependencyKind::DeviceExtension,
            })
            .unwrap_or(DependencyKind::Version)
    }

    pub fn find(&self, name: VkTyName) -> Option<&ExtensionInfo> {
        self.extensions.get(ExtensionName::Base { name })
    }

    pub fn extension_names_iter(&self) -> impl Iterator<Item = &str> + Clone {
        self.extensions
            .iter()
            .map(|e| e.extension_name.name_as_str())
    }

    pub fn extensions(&self) -> impl Iterator<Item = &ExtensionInfo> + Clone + use<'_> {
        self.extensions.iter()
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

        let traits = extensions.clone().map(|e| ExtensionTrait {
            name: e.extension_name,
        });

        // structs
        let instance_command_structs = extensions.clone().map(|e| ExtensionCommandStruct {
            name: e.extension_name,
            commands: &e.instance_command_names,
            command_method: "instance_commands",
        });
        let device_command_structs = extensions.clone().map(|e| ExtensionCommandStruct {
            name: e.extension_name,
            commands: &e.device_command_names,
            command_method: "device_commands",
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

        let macro_dependency_traits = extensions.clone().map(|e| DependencyTraits {
            info: e,
            all_extensions: self,
        });

        krs_quote_with!(tokens <-

            #[doc(hidden)]
            pub mod extension {
                {@* {@traits}}

                pub mod instance_command_structs {
                    use crate::LoadCommands;
                    {@* {@instance_command_structs}}
                }

                pub mod device_command_structs {
                    use crate::LoadCommands;
                    {@* {@device_command_structs}}
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
    command_method: &'a str,
}

impl krs_quote::ToTokens for ExtensionCommandStruct<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let command = self.commands.iter();
        let command_method = self.command_method.as_code();

        krs_quote_with!(tokens <-
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            pub struct {@name} {
                {@*
                    pub {@command}: crate::{@command},
                }
            }

            impl {@name} {
                #[allow(unused_variables)]
                pub fn load(loader: impl crate::FunctionLoader) -> std::result::Result<Self, crate::CommandLoadError> {
                    Ok(Self {
                        {@* {@command} : crate::{@command}::load(loader)?, }
                    })
                }
            }

            {@*
                impl<T> crate::has_command::{@command}<{@name}> for T
                    where T: super::{@name}
                {
                    fn {@command}(&self) -> crate::{@command} {
                        self.{@command_method}().{@command}
                    }
                }
            }
        );
    }
}

struct ExtensionTrait {
    name: ExtensionName,
}

impl krs_quote::ToTokens for ExtensionTrait {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;

        krs_quote_with!(tokens <-
            #[allow(non_camel_case_types)]
            pub unsafe trait {@name} {
                fn instance_commands(&self) -> &instance_command_structs::{@name} {
                    unreachable!();
                }

                fn device_commands(&self) -> &device_command_structs::{@name} {
                    unreachable!();
                }
            }

            unsafe impl<T> {@name} for T where T: crate::CommandWrapper<Commands: {@name}> {
                fn instance_commands(&self) -> &instance_command_structs::{@name} {
                    self.commands().instance_commands()
                }

                fn device_commands(&self) -> &device_command_structs::{@name} {
                    self.commands().device_commands()
                }
            }
        );
    }
}

struct DependencyTraits<'a> {
    info: &'a ExtensionInfo,
    all_extensions: &'a ExtensionCollection,
}

impl krs_quote::ToTokens for DependencyTraits<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.info.extension_name;

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

        let dependencies_to_tokens = |deps, level| {
            krs_quote::to_tokens_closure!(tokens {
                if let &Some(ref deps) = deps {
                    let solutions = SolutionCollection::new_simplified(deps, &self.all_extensions);

                    let message = format!("The {level} dependencies for `{}` are not satisfied", name.name_as_str());
                    let solution_text: Vec<_> = solutions.iter().map(|s| {
                        s.iter()
                            .map(|t| t.as_str())
                            .my_intersperse(" + ")
                            .collect::<String>()
                    }).collect();

                    let mut label;
                    if solution_text.len() == 1 {
                        label = format!("For {}, the {level} must enable {}", name.name_as_str(), solution_text[0]);
                    }
                    else {
                        label = format!("For {}, the {level} must enable one of:\n\t\t", name.name_as_str());
                        label.extend(solution_text.clone()
                            .iter().map(|s|s.as_str())
                            .my_intersperse("; or\n\t\t")
                        );
                    }

                    let notes = solution_text.iter().map(|s| format!("consider using: {s}"));

                    let bounds = solutions.iter().map(|s|
                        krs_quote::to_tokens_closure!(tokens {
                            let bounds = s.iter().copied();
                            krs_quote_with!(tokens <-
                                {@+* {@bounds}}
                            )
                        })
                    );

                    krs_quote_with!(tokens <-
                        use crate::dependency::*;

                        #[diagnostic::on_unimplemented(
                            message = {@message},
                            label = {@label},
                            {@,* note = {@notes}}
                        )]
                        #[marker]
                        pub trait HasDependency {}

                        {@*
                            impl<T> HasDependency for T where T: {@bounds} {}
                        }
                    )
                }
                else {
                    krs_quote_with!(tokens <-
                        #[marker]
                        pub trait HasDependency {}
                        impl<T> HasDependency for T {}
                    )
                }
            })
        };

        let instance_dependencies = dependencies_to_tokens(&instance_dependencies, "Instance");

        let device_dependencies = dependencies_to_tokens(&device_dependencies, "Device");

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
    pub fn new<'a>(term: &crate::vk_parse_visitor::Term) -> Self {
        match term {
            crate::vk_parse_visitor::Term::Single(name) => ExtensionName::Base {
                name: (*name).into(),
            },
            crate::vk_parse_visitor::Term::And(_) => ExtensionName::Extra {
                name: term.name().into(),
            },
            crate::vk_parse_visitor::Term::Or(_) => {
                panic!("extension name should not have or terms")
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

/// Command Names for a given extension
/// intended to generate code within a instance/device extension_names module
pub(crate) struct ExtensionInfo {
    extension_name: ExtensionName,
    instance_command_names: Vec<VkTyName>,
    device_command_names: Vec<VkTyName>,
    kind: ExtensionKind,
    dependencies: Option<DependencyTerm>,
    promoted_to: Option<VkTyName>,
}

impl ExtensionInfo {
    pub fn new(
        extension_name: ExtensionName,
        kind: ExtensionKind,
        promoted_to: Option<VkTyName>,
    ) -> Self {
        Self {
            extension_name,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
            kind,
            dependencies: Default::default(),
            promoted_to,
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
    pub fn promoted_to(&self) -> Option<VkTyName> {
        self.promoted_to
    }
    pub fn name(&self) -> ExtensionName {
        self.extension_name
    }
}
