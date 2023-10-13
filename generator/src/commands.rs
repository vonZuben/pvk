use krs_quote::krs_quote_with;

use crate::utils::{VecMap, VkTyName};

use crate::definitions;

#[derive(Default)]
pub struct Commands2 {
    function_pointers: VecMap<VkTyName, definitions::FunctionPointer>,
}

impl Commands2 {
    pub fn push(
        &mut self,
        name: impl Into<VkTyName>,
        function_pointer: definitions::FunctionPointer,
    ) {
        let name = name.into();
        self.function_pointers.push(name, function_pointer);
    }
}

impl krs_quote::ToTokens for Commands2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let function_pointers = self.function_pointers.iter();
        let commands = self.function_pointers.iter().map(|fptr| fptr.name);
        let command_names = self.function_pointers.iter().map(|fptr| fptr.name.as_str());
        krs_quote_with!( tokens <-
            {@* {@function_pointers}}

            {@*
                impl VulkanCommand for {@commands} {
                    const VK_NAME: *const c_char = concat!({@command_names}, '\0').as_ptr().cast();
                    unsafe fn new(ptr: PFN_vkVoidFunction) -> Self {
                        {@commands}(std::mem::transmute(ptr))
                    }
                }
            }

            /// Provider traits are intended to be automatically implemented for any type that contains the relevant function pointer
            /// See the feature and extension generated code for more details
            /// The Source generic parameter allows the provider trait to be implemented more than once so different version and extensions can provide the same command
            /// When a user creates their Instance or Device context, it is intended that they only choose a single provider of a command to avoid redundancy,
            /// and allow inference of the Source which is possible for the compiler if of there is only one Source
            /// (if multiple Sources are available in a context, a Source needs to be manually selected).
            /// For example, a command may have been provided by an extension, and then later promoted to a new core version.
            /// If upgrading the the new core version, the old extension can/should be removed from the context
            #[doc(hidden)]
            pub mod command {
                {@*
                    pub trait {@commands}<Source> {
                        fn {@commands}(&self) -> super::{@commands};
                    }
                }
            }
        );
    }
}
