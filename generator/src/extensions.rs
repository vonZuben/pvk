use krs_quote::krs_quote_with;

use crate::utils::VecMap;
use crate::utils::VkTyName;

use std::ops::{Deref, DerefMut};

// a collection of extensions
#[derive(Default)]
pub(crate) struct ExtensionCollection {
    extensions: VecMap<ExtensionName, ExtensionInfo>,
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

impl krs_quote::ToTokens for ExtensionCollection {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let extensions = self.extensions.iter();

        let e_name = extensions.clone().map(|e| e.extension_name);
        let instance_structs = extensions.clone().map(|e| ExtensionStruct {
            name: e.extension_name,
            commands: &e.instance_command_names,
        });
        let device_structs = extensions.clone().map(|e| ExtensionStruct {
            name: e.extension_name,
            commands: &e.device_command_names,
        });

        krs_quote_with!(tokens <-
            {@* {@extensions}}

            #[doc(hidden)]
            pub mod extension {
                pub mod instance {
                    use super::super::{LoadCommands, CommandLoadError, FunctionLoader};
                    {@* {@instance_structs}}
                    pub mod provider {
                        {@*
                            pub trait {@e_name} {
                                fn {@e_name}(&self) -> &super::{@e_name};
                            }
                        }
                    }
                }

                pub mod device {
                    use super::super::{LoadCommands, CommandLoadError, FunctionLoader};
                    {@* {@device_structs}}
                    pub mod provider {
                        {@*
                            pub trait {@e_name} {
                                fn {@e_name}(&self) -> &super::{@e_name};
                            }
                        }
                    }
                }
            }
        )
    }
}

struct ExtensionStruct<'a> {
    name: ExtensionName,
    commands: &'a Vec<VkTyName>,
}

impl krs_quote::ToTokens for ExtensionStruct<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let command = self.commands.iter();

        krs_quote_with!(tokens <-
            pub struct {@name} {
                {@* {@command}: super::super::{@command}, }
            }

            impl {@name} {
                pub fn load(loader: impl FunctionLoader) -> std::result::Result<Self, CommandLoadError> {
                    Ok(Self {
                        {@* {@command} : super::super::{@command}::load(loader)?, }
                    })
                }
            }

            {@*
                impl<T: provider::{@name}> super::super::command::{@command}<super::super::{@name}> for T {
                    fn {@command}(&self) -> super::super::{@command} {
                        self.{@name}().{@command}
                    }
                }
            }
        );
    }
}

// used to represent names of commands that are enabled by an extension and possible extra commands when other features/extensions are available
// base: base extension
// extra: feature or extension that adds more commands
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExtensionName {
    Base {
        name: VkTyName,
    },
    Extra {
        name: VkTyName,
        base: VkTyName,
        extra: VkTyName,
    },
}

impl ExtensionName {
    pub fn new(base: &str, extra: Option<&str>) -> Self {
        let base = VkTyName::new(base);
        match extra {
            Some(extra) => {
                let extra = VkTyName::new(extra);
                ExtensionName::Extra {
                    name: format!("{}_WITH_{}", base, extra).into(),
                    base,
                    extra,
                }
            }
            None => ExtensionName::Base { name: base },
        }
    }
    fn is_base(&self) -> bool {
        matches!(self, ExtensionName::Base { .. })
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
pub struct ExtensionInfo {
    extension_name: ExtensionName,
    instance_command_names: Vec<VkTyName>,
    device_command_names: Vec<VkTyName>,
    kind: ExtensionKind,
    required: Vec<VkTyName>,
}

impl ExtensionInfo {
    pub fn new(extension_name: ExtensionName, kind: ExtensionKind) -> Self {
        Self {
            extension_name,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
            kind,
            required: Default::default(),
        }
    }
    pub fn push_instance_command(&mut self, command: VkTyName) {
        self.instance_command_names.push(command);
    }
    pub fn push_device_command(&mut self, command: VkTyName) {
        self.device_command_names.push(command);
    }
    pub fn require<'a>(&mut self, require: impl Iterator<Item = &'a str>) {
        let require = require.map(|r| VkTyName::new(r));
        self.required.extend(require);
    }
}

impl krs_quote::ToTokens for ExtensionInfo {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let kind = self.kind;
        let extension_name = &self.extension_name;
        let raw_name = self.extension_name.name_as_str();
        let tmp;
        let required = match self.extension_name {
            ExtensionName::Base { .. } => self.required.as_slice(),
            ExtensionName::Extra { base, extra, .. } => {
                tmp = [base, extra];
                &tmp
            }
        };

        if self.extension_name.is_base() {
            krs_quote_with!(tokens <-
                #[derive(Debug)]
                pub struct {@extension_name};
                impl VulkanExtension for {@extension_name} {
                    type Require = ({@,* {@required}});
                    const VK_NAME: *const c_char = concat!({@raw_name}, '\0').as_ptr().cast();
                    type ExtensionType = {@kind};
                    type InstanceCommands = extension::instance::{@extension_name};
                    type DeviceCommands = extension::device::{@extension_name};
                }
            )
        } else {
            krs_quote_with!(tokens <-
                #[derive(Debug)]
                pub struct {@extension_name};
                impl VulkanExtensionExtras for {@extension_name} {
                    type Require = ({@,* {@required}});
                    type InstanceCommands = extension::instance::{@extension_name};
                    type DeviceCommands = extension::device::{@extension_name};
                }
            )
        }
    }
}
