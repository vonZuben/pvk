
use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

use crate::ctype::Ctype;

pub struct Cfield<'a> {
    name: &'a str,
    ty: Ctype<'a>,
}

impl<'a> Cfield<'a> {
    pub fn new(name: &'a str, ty: Ctype<'a>) -> Self {
        Self {
            name,
            ty,
        }
    }
}

impl ToTokens for Cfield<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name.as_code();
        let ty = &self.ty;

        quote!( #name : #ty ).to_tokens(tokens);
    }
}
