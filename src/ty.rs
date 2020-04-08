
#![allow(dead_code)]

use proc_macro2::TokenStream; // 1.0.9
use quote::{quote, ToTokens}; // 1.0.3

use std::default::Default;

use crate::utils::StrAsCode;

pub enum Reference {
    Yes,
    No,
}

impl Default for Reference {
    fn default() -> Self {
        Reference::No
    }
}

impl From<bool> for Reference {
    fn from(b: bool) -> Self {
        match b {
            true => Reference::Yes,
            false => Reference::No,
        }
    }
}

impl ToTokens for Reference {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Reference::*;
        match &self {
            Yes => quote!(&).to_tokens(tokens),
            No => {}
        }
    }
}

#[derive(Default)]
pub struct Lifetime {
    l: String,
}

impl<S: ToString> From<S> for Lifetime {
    fn from(s: S) -> Self {
        Lifetime{ l: s.to_string() }
    }
}

impl ToTokens for Lifetime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.l.as_code().to_tokens(tokens);
    }
}

pub enum Mutable {
    Yes,
    No,
}

impl Default for Mutable {
    fn default() -> Self {
        Mutable::No
    }
}

impl From<bool> for Mutable {
    fn from(b: bool) -> Self {
        match b {
            true => Mutable::Yes,
            false => Mutable::No,
        }
    }
}

impl ToTokens for Mutable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Mutable::*;
        match &self {
            Yes => quote!(mut).to_tokens(tokens),
            No => {}
        }
    }
}

pub enum Pointer {
    Mut,
    Const,
    None,
}

impl Default for Pointer {
    fn default() -> Self {
        Pointer::None
    }
}

impl ToTokens for Pointer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Pointer::*;
        match &self {
            Mut => quote!(*mut).to_tokens(tokens),
            Const => quote!(*const).to_tokens(tokens),
            None => {}
        }
    }
}

pub enum Core {
    Ty(Box<Ty>),
    Basetype(String),
}

impl Default for Core {
    fn default() -> Self {
        Core::Basetype(String::new())
    }
}

impl<S: ToString> From<S> for Core {
    fn from(s: S) -> Self {
        Core::Basetype(s.to_string())
    }
}

impl From<Ty> for Core {
    fn from(ty: Ty) -> Self {
        Core::Ty(Box::new(ty))
    }
}

impl ToTokens for Core {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Core::*;
        match &self {
            Ty(ty) => ty.to_tokens(tokens),
            Basetype(name) => name.as_code().to_tokens(tokens),
        }
    }
}

#[derive(Default)]
pub struct TypeParams {
    params: Vec<Ty>,
}

impl ToTokens for TypeParams {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let p = self.params.iter();
        if !self.params.is_empty() {
            quote!( < #(#p),* > ).to_tokens(tokens)
        }
    }
}

pub enum Array {
    Static,
    Dynamic(String), // holds actual size
    None,
}

impl Default for Array {
    fn default() -> Self {
        Array::None
    }
}

impl Array {
    fn dynamic(s: impl ToString) -> Self {
        Array::Dynamic(s.to_string())
    }
}

#[derive(Default)]
pub struct Ty {
    pub reference: Reference,
    pub lifetime: Lifetime,
    pub mutable: Mutable,
    pub pointer: Pointer,
    pub core: Core,
    pub type_params: TypeParams,
    pub array: Array,
}

impl ToTokens for Ty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let reference = &self.reference;
        let lifetime = &self.lifetime;
        let mutable = &self.mutable;
        let pointer = &self.pointer;
        let core = &self.core;
        let params = &self.type_params;

        let ty = quote!( #reference #lifetime #mutable #pointer #core #params );

        use Array::*;
        match &self.array {
            Static => quote!([ty]).to_tokens(tokens),
            Dynamic(size) => {
                let size = size.as_code();
                quote!([#ty;#size]).to_tokens(tokens);
            }
            None => ty.to_tokens(tokens),
        }
    }
}

impl From<Reference> for Ty {
    fn from(r: Reference) -> Self {
        Ty::new().reference(r)
    }
}

impl From<Lifetime> for Ty {
    fn from(l: Lifetime) -> Self {
        Ty::new().lifetime(l)
    }
}

impl From<Mutable> for Ty {
    fn from(m: Mutable) -> Self {
        Ty::new().mutable(m)
    }
}

impl From<Pointer> for Ty {
    fn from(p: Pointer) -> Self {
        Ty::new().pointer(p)
    }
}

impl From<Core> for Ty {
    fn from(c: Core) -> Self {
        Ty::new().core(c)
    }
}

impl Ty {
    fn new() -> Self {
        Default::default()
    }
    fn reference(mut self, r: impl Into<Reference>) -> Self {
        self.reference = r.into();
        self
    }
    fn lifetime(mut self, l: impl Into<Lifetime>) -> Self {
        self.lifetime = l.into();
        self
    }
    fn mutable(mut self, m: impl Into<Mutable>) -> Self {
        self.mutable = m.into();
        self
    }
    fn pointer(mut self, p: Pointer) -> Self {
        self.pointer = p;
        self
    }
    fn core(mut self, c: impl Into<Core>) -> Self {
        self.core = c.into();
        self
    }
    fn array(mut self, a: Array) -> Self {
        self.array = a;
        self
    }
    fn param(mut self, p: impl Into<Ty>) -> Self {
        self.type_params.params.push(p.into());
        self
    }
    fn push_param(&mut self, p: impl Into<Ty>) {
        self.type_params.params.push(p.into());
    }
}

pub fn test() {
    let ty2 = Ty::new()
        .reference(true)
        .lifetime("'r")
        .pointer(Pointer::Const)
        .core("Ref")
        .array(Array::dynamic("10"))
        .param(Lifetime::from("'a"))
        .param(Ty::new().core("Hello").param(Lifetime::from("'a")));

    let ty3 = Ty::new()
        .reference(true)
        .core(ty2);

    println!("{}", quote!(#ty3));
}
