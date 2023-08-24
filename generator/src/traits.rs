use krs_quote::{krs_quote_with, ToTokens};

pub struct VulkanCommand;

impl ToTokens for VulkanCommand {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        krs_quote_with!(tokens <-
            pub trait VulkanCommand : Copy + Sized {
                const VK_NAME: *const c_char;
                unsafe fn new(ptr: PFN_vkVoidFunction) -> Self;
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
            pub struct InstanceExtension;
            pub struct DeviceExtension;

            pub trait VulkanExtension {
                /// This is intended to be an Hlist representing all other extension prerequisites for this extension
                type Require;

                const VK_NAME: *const c_char;

                type ExtensionType;

                type InstanceCommands;
                type DeviceCommands;
            }

            pub trait VulkanExtensionExtras {
                /// This is intended to be an Hlist representing all other extension prerequisites for this extension
                type Require;

                type InstanceCommands;
                type DeviceCommands;
            }
        )
    }
}