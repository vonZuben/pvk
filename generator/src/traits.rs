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