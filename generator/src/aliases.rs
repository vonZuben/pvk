use krs_quote::{my_quote, my_quote_with};

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

impl<'a, I: Iterator<Item=TypeDef> + Clone> krs_quote::ToTokens for CmdAliasNames<I> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.alias_defs.clone().map(|td|td.name);
        let alias = self.alias_defs.clone().map(|td|td.ty);

        my_quote_with!( tokens {
            macro_rules! use_cmd_alias_pairs {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* {@,* {@name} = {@alias} } );
                }
            }
        });
    }
}