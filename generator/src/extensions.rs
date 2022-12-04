use krs_quote::my_quote_with;

use crate::utils::{VkTyName, StrAsCode};

// used to represent names of commands that are enabled by an extension and possible extra commands when other features/extensions are available
// base: base extension
// extra: feature or extension that adds more commands
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExtensionName {
    Noraml(VkTyName),
    Extended(VkTyName, VkTyName),
}

impl ExtensionName {
    pub fn new(base: &str, extra: Option<&str>) -> Self {
        let base = VkTyName::new(base);
        match extra {
            Some(extra) => {
                let extra = VkTyName::new(extra);
                ExtensionName::Extended(base, extra)
            }
            None => {
                ExtensionName::Noraml(base)
            }
        }
    }
}

impl krs_quote::ToTokens for ExtensionName {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        match self {
            ExtensionName::Noraml(name) => {
                my_quote_with!(tokens {{@name}});
            }
            ExtensionName::Extended(base, extend) => {
                let name = format!("{}_WITH_{}", base, extend).as_code();
                my_quote_with!(tokens {{@name}});
            }
        }
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
            Self::Instance => my_quote_with!(tokens {INSTANCE}),
            Self::Device => my_quote_with!(tokens {DEVICE}),
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
        let require = require.map(|r|VkTyName::new(r));
        self.required.extend(require);
    }
}

impl krs_quote::ToTokens for ExtensionInfo {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let kind = self.kind;
        let extension_name = &self.extension_name;
        let instance_command_names = &self.instance_command_names;
        let device_command_names = &self.device_command_names;
        let all_commands_names = instance_command_names.iter().chain(device_command_names.iter());
        let load = match &self.extension_name {
            ExtensionName::Noraml(name) => Some(&**name),
            ExtensionName::Extended(_, _) => None,
        };
        let required: Vec<_> = match self.extension_name {
            ExtensionName::Noraml(_name) => self.required.iter().copied().collect(),
            ExtensionName::Extended(base, _) => std::iter::once(base).collect(),
        };
        my_quote_with!( tokens {
            macro_rules! {@extension_name} {
                ( @KIND $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@kind} );
                };
                ( @INSTANCE_COMMANDS $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@,* {@instance_command_names}} );
                };
                ( @DEVICE_COMMANDS $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@,* {@device_command_names}} );
                };
                ( @ALL_COMMANDS $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@,* {@all_commands_names}} );
                };
                ( @REQUIRE $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@load} ; {@,* {@required}} );
                };
            }
        });
    }
}

// =================================================================
/// list of all existing Vulkan extensions
#[derive(Default)]
pub struct VulkanExtensionNames {
    extensions: Vec<ExtensionName>,
}

impl VulkanExtensionNames {
    pub fn push_extension(&mut self, extension_name: ExtensionName) {
        self.extensions.push(extension_name);
    }
}

impl krs_quote::ToTokens for VulkanExtensionNames {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let extension_names = &self.extensions;
        my_quote_with!( tokens {
            macro_rules! use_all_vulkan_extension_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@,* {@extension_names}} );
                }
            }
        });
    }
}