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

pub struct EnumTraits;

impl ToTokens for EnumTraits {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        krs_quote_with!(tokens <-
            pub trait VkEnum : Sized {
                fn from_variant_type<V: VkEnumVariant<Enum=Self>>(_: V) -> Self;
            }
            /// All Vk Enums are C enums, which means they are i32
            pub trait VkEnumVariant {
                type Enum;
                const VARIANT: i32;
            }

            #[derive(Copy, Clone)]
            pub struct ConstVariant<V, T: VkEnumVariant<Enum = V>>(std::marker::PhantomData<T>, std::marker::PhantomData<V>);

            impl<V, T: VkEnumVariant<Enum = V>> ConstVariant<V, T> {
                pub const fn new() -> Self {
                    Self(std::marker::PhantomData, std::marker::PhantomData)
                }
                pub const fn is<O: VkEnumVariant<Enum = V> + Copy>(self, _: O) -> bool {
                    T::VARIANT == O::VARIANT
                }
            }

            #[macro_export]
            macro_rules! const_enum {
                ( $ty:ident ) => {
                    $crate::ConstVariant::<_, $ty>::new()
                }
            }
        )
    }
}