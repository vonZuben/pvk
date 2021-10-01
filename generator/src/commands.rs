
use quote::{quote, ToTokens};

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::{utils::*};
use crate::utils;

use crate::utils::VecMap;

use std::collections::HashMap;

use crate::definitions;

#[derive(Default)]
pub struct Commands2<'a> {
    function_pointers: VecMap<&'a str, definitions::FunctionPointer<'a>>,
}

// impl<'a, I: IntoIterator<Item=definitions::FunctionPointer<'a>>> From<I> for Commands2<'a> {
//     fn from(i: I) -> Self {
//         Self {
//             function_pointers: i.into_iter().collect(),
//         }
//     }
// }

impl<'a> Commands2<'a> {
    pub fn push(&mut self, name: &'a str, function_pointer: definitions::FunctionPointer<'a>) {
        self.function_pointers.push(name, function_pointer);
    }
    pub fn contains(&self, name: &str) -> bool {
        self.function_pointers.get(name).is_some()
    }
}

impl ToTokens for Commands2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let function_pointers = self.function_pointers.iter();
        let commands = self.function_pointers.iter().map(|fptr|fptr.name.as_code());
        let command_names = self.function_pointers.iter().map(|fptr|fptr.name);
        quote!(
            #(#function_pointers)*
            macro_rules! use_command_function_pointer_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#commands -> #command_names);* );
                }
            }
        ).to_tokens(tokens);
    }
}