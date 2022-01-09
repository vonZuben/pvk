
use quote::{ToTokens, quote};

use crate::{definitions::TypeDef, utils::{StrAsCode, VecMap}};

pub struct CmdAliasNames<I> {
    alias_defs: I,
}

impl<I> CmdAliasNames<I> {
    pub fn new(i: I) -> Self {
        Self {
            alias_defs: i,
        }
    }
}

impl<'a, I: Iterator<Item=TypeDef> + Clone> ToTokens for CmdAliasNames<I> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.alias_defs.clone().map(|td|td.name);
        let alias = self.alias_defs.clone().map(|td|td.ty);

        quote!(
            macro_rules! use_cmd_alias_pairs {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #( #name = #alias ),* );
                }
            }
        ).to_tokens(tokens);
    }
}