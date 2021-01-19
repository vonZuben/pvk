
#![allow(dead_code)]

use proc_macro2::TokenStream; // 1.0.9
use quote::{quote, ToTokens}; // 1.0.3

use std::default::Default;

use crate::utils::StrAsCode;
use crate::utils;

pub enum Reference {
    True(Lifetime),
    False,
}

impl Default for Reference {
    fn default() -> Self {
        Reference::False
    }
}

impl From<bool> for Reference {
    fn from(b: bool) -> Self {
        match b {
            true => Reference::True(Lifetime::None),
            false => Reference::False,
        }
    }
}

impl From<Lifetime> for Reference {
    fn from(lifetime: Lifetime) -> Self {
        Self::True(lifetime)
    }
}

impl From<&str> for Reference {
    fn from(lifetime: &str) -> Self {
        Lifetime::from(lifetime).into()
    }
}

impl ToTokens for Reference {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Reference::*;
        match &self {
            True(lifetime) => quote!( & #lifetime ).to_tokens(tokens),
            False => {}
        }
    }
}

pub enum Lifetime {
    None,
    Anonymous,
    Named(String),
}

impl<S: ToString> From<S> for Lifetime {
    fn from(s: S) -> Self {
        Lifetime::named(s)
    }
}

impl Lifetime {
    fn named(s: impl ToString) -> Self {
        let s = s.to_string();
        assert!(s.starts_with('\''));
        Lifetime::Named(s)
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Lifetime::None
    }
}

impl From<utils::WithLifetime<'_>> for Lifetime {
    fn from(lt: utils::WithLifetime) -> Self {
        use utils::WithLifetime;
        match lt {
            WithLifetime::Yes(lifetime) => lifetime.into(),
            WithLifetime::No => Lifetime::None,
        }
    }
}

impl ToTokens for Lifetime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        //self.l.as_code().to_tokens(tokens);
        match self {
            Lifetime::None => {}
            Lifetime::Anonymous => "'_".as_code().to_tokens(tokens),
            Lifetime::Named(name) => name.as_code().to_tokens(tokens),
    }
}
}

pub enum Mutable {
    True,
    False,
}

impl Default for Mutable {
    fn default() -> Self {
        Mutable::False
    }
}

impl From<bool> for Mutable {
    fn from(b: bool) -> Self {
        match b {
            true => Mutable::True,
            false => Mutable::False,
        }
    }
}

impl ToTokens for Mutable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Mutable::*;
        match &self {
            True => quote!(mut).to_tokens(tokens),
            False => {}
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

#[derive(Default)]
pub struct Basetype {
    pub name: Option<String>,
    pub generics: Generics,
}

impl<S: ToString> From<S> for Basetype {
    fn from(s: S) -> Self {
        Basetype {
            name: Some(s.to_string()),
            generics: Generics::default(),
        }
    }
}

impl ToTokens for Basetype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_ref().expect("error: never gave Basetype a name");
        assert!(!name.is_empty());
        let name = name.as_code();
        let generics = &self.generics;
        quote!( #name #generics ).to_tokens(tokens);
    }
}

pub enum Core {
    Basetype(Basetype),
    Array(Box<Array>),
}

impl Default for Core {
    fn default() -> Self {
        Core::Basetype(Basetype::default())
    }
}

impl Core {
    fn push_lifetime_param(&mut self, l: impl Into<Lifetime>) {
        match self {
            Core::Basetype(basetype) => basetype.generics.push_lifetime_param(l),
            _ => panic!("can only push params when core is Basetype"),
        }
    }
    fn push_type_param(&mut self, t: impl Into<Ty>) {
        match self {
            Core::Basetype(basetype) => basetype.generics.push_type_param(t),
            _ => panic!("can only push params when core is Basetype"),
        }
    }
    pub fn basetype(b: impl Into<Basetype>) -> Self {
        Core::Basetype(b.into())
    }
    pub fn array(a: impl Into<Array>) -> Self {
        Core::Array(Box::new(a.into()))
    }
}

impl From<Basetype> for Core {
    fn from(basetype: Basetype) -> Self {
        Core::Basetype(basetype)
    }
}

impl From<Array> for Core {
    fn from(array: Array) -> Self {
        Core::Array( Box::new(array) )
    }
}

impl ToTokens for Core {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Core::*;
        match &self {
            Basetype(basetype) => basetype.to_tokens(tokens),
            Array(array) => array.to_tokens(tokens),
        }
    }
}

#[derive(Default)]
pub struct Generics {
    lifetime_params: Vec<Lifetime>,
    type_params: Vec<Ty>,
}

impl Generics {
    fn push_lifetime_param(&mut self, l: impl Into<Lifetime>) {
        self.lifetime_params.push(l.into());
    }
    fn push_type_param(&mut self, t: impl Into<Ty>) {
        self.type_params.push(t.into());
    }
}

impl ToTokens for Generics {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if ! self.lifetime_params.is_empty() || ! self.type_params.is_empty() {
            let lifetime_params = self.lifetime_params.iter().map(|i|{i as &dyn ToTokens});
            let type_params = self.type_params.iter().map(|i|{i as &dyn ToTokens});
            let iter = lifetime_params.chain(type_params);
            quote!( < #(#iter),* > ).to_tokens(tokens)
        }
    }
}

pub struct Array {
    ty: Ty,
    array_type: ArrayType,
}

impl ToTokens for Array {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = &self.ty;
        match &self.array_type {
            ArrayType::Slice => quote!( [#ty] ).to_tokens(tokens),
            ArrayType::Array(size) => {
                let size = size.as_code();
                quote!( [#ty;#size] ).to_tokens(tokens)
            }
        }
    }
}

impl Array {
    fn size(mut self, array_type: ArrayType) -> Self {
        self.array_type = array_type;
        self
    }
}

impl From<Ty> for Array {
    fn from(ty: Ty) -> Self {
        Array {
            ty,
            array_type: ArrayType::Slice,
        }
    }
}

pub enum ArrayType {
    Array(String), // storeing size as string for code gen reasons
    Slice,
}

impl ArrayType {
    pub fn array(size: impl ToString) -> Self {
        ArrayType::Array(size.to_string())
    }
    pub fn slice() -> Self {
        ArrayType::Slice
    }
}

#[derive(Default)]
pub struct Ty {
    reference: Reference,
    mutable: Mutable,
    pointer: Vec<Pointer>,
    core: Core,
}

impl ToTokens for Ty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let reference = &self.reference;
        let mutable = &self.mutable;
        let pointer = &self.pointer;
        let core = &self.core;

        quote!( #reference #mutable #(#pointer)* #core ).to_tokens(tokens);
    }
}



impl Ty {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn reference(mut self, r: impl Into<Reference>) -> Self {
        self.reference = r.into();
        self
    }
    pub fn mutable(mut self, m: impl Into<Mutable>) -> Self {
        self.mutable = m.into();
        self
    }
    pub fn pointer(mut self, p: Pointer) -> Self {
        self.pointer.push(p);
        self
    }
    pub fn basetype(mut self, c: impl Into<Basetype>) -> Self {
        self.core(c.into())
    }
    pub fn array(mut self, ty: Ty, array_type: ArrayType) -> Self {
        self.core( Array{ ty, array_type } )
    }
    pub fn to_array(self, array_type: ArrayType) -> Self {
        Ty::new().array(self, array_type)
    }
    pub fn lifetime_param(mut self, l: impl Into<Lifetime>) -> Self {
        self.core.push_lifetime_param(l);
        self
    }
    pub fn type_param(mut self, p: impl Into<Ty>) -> Self {
        self.core.push_type_param(p);
        self
    }
    pub fn core(mut self, core: impl Into<Core>) -> Self {
        self.core = core.into();
        self
    }
}

pub struct Field {
    pub name: String,
    pub ty: Ty,
}

impl Field {
    pub fn new(name: impl ToString, ty: impl Into<Ty>) -> Self {
        Field {
            name: name.to_string(),
            ty: ty.into(),
        }
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_code();
        let ty = &self.ty;
        quote!( #name : #ty ).to_tokens(tokens);
    }
}

pub fn test() {
    let ty2 = Ty::new()
        .reference("'r")
        .pointer(Pointer::Const)
        .basetype("Ref")
        .lifetime_param("'a")
        .type_param(Ty::new().basetype("Hello").lifetime_param("'a"))
        .to_array(ArrayType::Slice)
        .reference(true);

    println!("{}", quote!(#ty2));
}
