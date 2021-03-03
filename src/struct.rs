
use proc_macro2::TokenStream; // 1.0.9
use quote::{quote, ToTokens}; // 1.0.3

use crate::{ty::*, utils::StrAsCode};

// only supports normal structs (not unit or tuple structs)

#[derive(Default)]
pub struct Struct {
    name: String,
    generics: Generics,
    fields: Vec<Field>,
    attributes: Vec<TokenStream>,
    public: bool,
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_code();
        let generics = &self.generics;
        let fields = &self.fields;
        let attributes = &self.attributes;

        let public = if self.public {
            Some(quote!(pub))
        }
        else {
            None
        };

        quote!(
            #(#attributes)*
            #public struct #name #generics {
                #(#fields,)*
            }
        ).to_tokens(tokens);
    }
}

impl Struct {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            .. Self::default()
        }
    }
    pub fn lifetime_param(mut self, l: impl Into<Lifetime>) -> Self {
        self.generics.push_lifetime_param(l);
        self
    }
    #[allow(unused)]
    pub fn type_param(mut self, t: impl ToTokens) -> Self {
        self.generics.push_type_param(t);
        self
    }
    pub fn fields(mut self, f: impl IntoIterator<Item=Field>) -> Self {
        self.fields = f.into_iter().collect();
        self
    }
    pub fn attribute(mut self, a: TokenStream) -> Self {
        self.attributes.push(a);
        self
    }
    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }
}