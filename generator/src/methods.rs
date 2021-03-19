use proc_macro2::TokenStream; // 1.0.9
use quote::{quote, ToTokens}; // 1.0.3

use crate::{ty::*, utils::StrAsCode};

#[derive(Debug, Clone, Copy)]
pub enum Visiblity {
    Public,
    Private,
}

impl Default for Visiblity {
    fn default() -> Self {
        Self::Private
    }
}

impl ToTokens for Visiblity {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Private => {},
            Self::Public => quote!(pub).to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Safty {
    Safe,
    Unsafe,
}

impl Default for Safty {
    fn default() -> Self {
        Self::Safe
    }
}

impl ToTokens for Safty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Safe => {},
            Self::Unsafe => quote!(unsafe).to_tokens(tokens),
        }
    }
}

#[derive(Default)]
pub struct Return {
    ty: Option<Ty>,
}

impl From<Ty> for Return {
    fn from(ty: Ty) -> Self {
        Self {
            ty: Some(ty)
        }
    }
}

impl ToTokens for Return {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.ty.as_ref() {
            Some(ty) => quote!{ -> #ty }.to_tokens(tokens),
            None => {}
        }
    }
}

#[derive(Default)]
pub struct Body {
    body: TokenStream,
}

impl Body {
    fn push_code(&mut self, code: impl ToTokens) {
        code.to_tokens(&mut self.body);
    }
}

impl ToTokens for Body {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let body = &self.body;
        quote!(#body).to_tokens(tokens);
    }
}

#[derive(Default)]
pub struct Method {
    vis: Visiblity,
    safty: Safty,
    name: String,
    generics: Generics,
    fields: Vec<Field>,
    ret: Return,
    body: Body,
}

impl Method {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }
    pub fn public(mut self) -> Self {
        self.vis = Visiblity::Public;
        self
    }
    pub fn not_safe(mut self) -> Self {
        self.safty = Safty::Unsafe;
        self
    }
    pub fn lifetime_param(mut self, l: impl Into<Lifetime>) -> Self {
        self.generics.push_lifetime_param(l);
        self
    }
    pub fn type_param(mut self, t: impl ToTokens) -> Self {
        self.generics.push_type_param(t);
        self
    }
    pub fn field(mut self, f: impl Into<Field>) -> Self {
        self.fields.push(f.into());
        self
    }
    pub fn fields(mut self, f: impl IntoIterator<Item=Field>) -> Self {
        self.fields.extend(f.into_iter());
        self
    }
    pub fn ret(mut self, ret: impl Into<Return>) -> Self {
        self.ret = ret.into();
        self
    }
    pub fn body_code(mut self, code: impl ToTokens) -> Self {
        self.body.push_code(code);
        self
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = self.vis;
        let safty = self.safty;
        let name = self.name.as_code();
        let generics = &self.generics;
        let fields = &self.fields;
        let ret = &self.ret;
        let body = &self.body;

        quote! (
            #vis #safty fn #name #generics ( #(#fields)* ) #ret {
                #body
            }
        ).to_tokens(tokens);
    }
}

#[derive(Default)]
pub struct MethodCall {
    name: String,
    fields: Vec<TokenStream>,
}

impl MethodCall {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
    pub fn field(mut self, f: impl StrAsCode) -> Self {
        self.fields.push(f.as_code());
        self
    }
    pub fn fields<'a>(mut self, f: impl IntoIterator<Item=&'a dyn StrAsCode>) -> Self {
        self.fields.extend(f.into_iter().map(|f|f.as_code()));
        self
    }
}

impl ToTokens for MethodCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_code();
        let fields = &self.fields;

        quote!(
            #name(#(#fields),*)
        ).to_tokens(tokens);
    }
}