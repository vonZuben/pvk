use krs_quote::krs_quote_with;

use crate::utils::{VecMap, VkTyName};

use crate::types;

#[derive(Default)]
pub struct Commands2 {
    commands: VecMap<VkTyName, Command>,
}

impl Commands2 {
    pub fn push(&mut self, name: impl Into<VkTyName>, function_pointer: types::FunctionPointer) {
        let name = name.into();
        self.commands.push(
            name,
            Command {
                function_pointer,
                enabled: false,
            },
        );
    }

    pub fn enable_command(&mut self, name: VkTyName) {
        let cmd = self
            .commands
            .get_mut(name)
            .expect("trying to enable command that does not exist");
        cmd.enabled = true;
    }
}

struct Command {
    function_pointer: types::FunctionPointer,
    enabled: bool,
}

impl krs_quote::ToTokens for Commands2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let function_pointers = self.commands.iter().filter_map(|cmd| {
            if cmd.enabled {
                Some(&cmd.function_pointer)
            } else {
                None
            }
        });

        let commands = function_pointers.clone().map(|fptr| fptr.name);
        let command_names = function_pointers.clone().map(|fptr| fptr.name.as_str());

        krs_quote_with!( tokens <-
            {@* {@function_pointers}}

            {@*
                impl VulkanCommand for {@commands} {
                    const VK_NAME: *const c_char = concat!({@command_names}, '\0').as_ptr().cast();
                    unsafe fn new(ptr: PFN_vkVoidFunction) -> Self {
                        {@commands}(ptr)
                    }
                }
            }

            /// Traits to provide the respective command, and macros to implement the same
            ///
            /// For the macros to work, there will need to be a path to this module from
            /// the root of the crate containing the generated code.
            /// Also, the command types themselves will need to be at the root of the crate
            #[doc(hidden)]
            pub mod has_command {
                {@*
                    pub trait {@commands} {
                        #[allow(non_snake_case)]
                        fn {@commands}(&self) -> super::{@commands};
                    }

                    #[doc(hidden)]
                    #[macro_export]
                    macro_rules! {@commands} {
                        ($target:ident $provider:ident) => {
                            #[allow(non_snake_case)]
                            impl $crate::has_command::{@commands} for $target {
                                fn {@commands}(&self) -> $crate::{@commands} {
                                    self.$provider.{@commands}
                                }
                            }
                        };
                    }
                }
            }
        );
    }
}
