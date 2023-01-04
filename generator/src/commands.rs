use krs_quote::krs_quote_with;

use crate::utils::{VkTyName, VecMap};

use crate::definitions;

#[derive(Default)]
pub struct Commands2 {
    function_pointers: VecMap<VkTyName, definitions::FunctionPointer>,
}

impl Commands2 {
    pub fn push(&mut self, name: impl Into<VkTyName>, function_pointer: definitions::FunctionPointer) {
        let name = name.into();
        self.function_pointers.push(name, function_pointer);
    }
    pub fn contains(&self, name: VkTyName) -> bool {
        self.function_pointers.get(name).is_some()
    }
}

impl krs_quote::ToTokens for Commands2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let function_pointers = self.function_pointers.iter();
        let commands = self.function_pointers.iter().map(|fptr|fptr.name);
        let command_names = self.function_pointers.iter().map(|fptr|fptr.name.as_str());
        krs_quote_with!( tokens <-
            {@* {@function_pointers}}
            macro_rules! use_command_function_pointer_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@;* {@commands} -> {@command_names} } );
                }
            }
        );
    }
}