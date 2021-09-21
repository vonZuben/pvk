
use quote::{quote, ToTokens};

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::{utils::*};
use crate::utils;

use std::collections::HashMap;

use crate::definitions;

#[derive(Default)]
pub struct Commands2<'a> {
    function_pointers: Vec<definitions::FunctionPointer<'a>>,
}

// impl<'a, I: IntoIterator<Item=definitions::FunctionPointer<'a>>> From<I> for Commands2<'a> {
//     fn from(i: I) -> Self {
//         Self {
//             function_pointers: i.into_iter().collect(),
//         }
//     }
// }

impl<'a> Commands2<'a> {
    pub fn push(&mut self, function_pointer: definitions::FunctionPointer<'a>) {
        self.function_pointers.push(function_pointer);
    }
}

impl ToTokens for Commands2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let function_pointers = &self.function_pointers;
        let commands = self.function_pointers.iter().map(|fptr|fptr.name.as_code());
        quote!(
            #(#function_pointers)*
            macro_rules! use_command_function_pointer_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#commands),* );
                }
            }
        ).to_tokens(tokens);
    }
}