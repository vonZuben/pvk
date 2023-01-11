use krs_quote::{krs_quote_with, ToTokens};

pub struct VulkanCommand;

impl ToTokens for VulkanCommand {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        krs_quote_with!(tokens <-
            pub trait VulkanCommand : Copy + Sized {
                const VK_NAME: *const c_char;
            }
        )
    }
}

pub struct VulkanVersion;

impl ToTokens for VulkanVersion {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        krs_quote_with!(tokens <-
            pub trait VulkanVersion {
                const VersionTriple: (u32, u32, u32);
                type InstanceCommands;
                type DeviceCommands;
                type EntryCommands;
            }
        )
    }
}

pub struct VulkanExtension;

impl ToTokens for VulkanExtension {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        krs_quote_with!(tokens <-
            pub trait Extension {
                /// This is intended to be an Hlist representing all other extension prerequisites for this extension
                type Require;
                /// Represent if this extension needs to load (i.e. some implementors of this trait only represent additional optional commands which otherwise require a base extension)
                type Load;
                const LoadThis: Self::Load;

                type ExtensionType;

                type Commands : commands::LoadCommands;
                fn load_extension_commands(f: impl commands::FunctionLoader) -> Result<Self::Commands, commands::LoadError> {
                    <Self::Commands as commands::LoadCommands>::load(f)
                }
            }
        )
    }
}