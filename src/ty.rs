
#![allow(dead_code)]

use proc_macro2::TokenStream; // 1.0.9
use quote::{quote, ToTokens}; // 1.0.3

use std::default::Default;

use crate::utils::StrAsCode;
use crate::utils;

pub enum Reference {
    True,
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
            true => Reference::True,
            false => Reference::False,
        }
    }
}

impl ToTokens for Reference {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Reference::*;
        match &self {
            True => quote!(&).to_tokens(tokens),
            False => {}
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

impl From<utils::WithLifetime<'_>> for Lifetime {
    fn from(lt: utils::WithLifetime) -> Self {
        use utils::WithLifetime;
        match lt {
            WithLifetime::Yes(lifetime) => lifetime.into(),
            WithLifetime::No => "".into(),
        }
    }
}

impl ToTokens for Lifetime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.l.as_code().to_tokens(tokens);
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

pub struct Basetype {
    pub name: String,
    pub params: Option<TypeParams>,
}

impl Basetype {
    fn push_param(&mut self, p: impl Into<Ty>) {
        match self.params {
            None => {
                let mut tp = TypeParams::default();
                tp.push(p);
                self.params = Some(tp);
            }
            Some(ref mut tp) => tp.push(p),
        }
    }
}

impl Default for Basetype {
    fn default() -> Self {
        "".into()
    }
}

impl<S: ToString> From<S> for Basetype {
    fn from(s: S) -> Self {
        Basetype {
            name: s.to_string(),
            params: None,
        }
    }
}

impl ToTokens for Basetype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_code();
        let params = &self.params;
        quote!( #name #params ).to_tokens(tokens);
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
    pub fn push_param(&mut self, p: impl Into<Ty>) {
        match self {
            Core::Basetype(basetype) => basetype.push_param(p),
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
pub struct TypeParams(Vec<Ty>);

impl TypeParams {
    pub fn push(&mut self, p: impl Into<Ty>) {
        self.0.push(p.into());
    }
}

impl ToTokens for TypeParams {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let p = self.0.iter();
        if !self.0.is_empty() {
            quote!( < #(#p),* > ).to_tokens(tokens)
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
    lifetime: Lifetime,
    mutable: Mutable,
    pointer: Vec<Pointer>,
    core: Core,
}

impl ToTokens for Ty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let reference = &self.reference;
        let lifetime = &self.lifetime;
        let mutable = &self.mutable;
        let pointer = &self.pointer;
        let core = &self.core;

        quote!( #reference #lifetime #mutable #(#pointer)* #core ).to_tokens(tokens);
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
        let mut ty = Ty::new();
        ty.core = c;
        ty
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
    pub fn lifetime(mut self, l: impl Into<Lifetime>) -> Self {
        self.lifetime = l.into();
        self
    }
    pub fn set_lifetime(&mut self, l: impl Into<Lifetime>) {
        self.lifetime = l.into();
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
    pub fn param(mut self, p: impl Into<Ty>) -> Self {
        self.core.push_param(p);
        self
    }
    pub fn push_param(&mut self, p: impl Into<Ty>) {
        self.core.push_param(p);
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
        .reference(true)
        .lifetime("'r")
        .pointer(Pointer::Const)
        .basetype("Ref")
        .param(Lifetime::from("'a"))
        .param(Ty::new().basetype("Hello").param(Lifetime::from("'a")))
        .to_array(ArrayType::Slice)
        .reference(true);

    println!("{}", quote!(#ty2));
}
