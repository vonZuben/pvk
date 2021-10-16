
use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

use crate::utils::case;

#[derive(Copy, Clone)]
enum Visability {
    Private,
    Public,
}

impl Default for Visability {
    fn default() -> Self {
        Visability::Private
    }
}

impl ToTokens for Visability {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Visability::*;
        match self {
            Private => {}
            Public => quote!(pub).to_tokens(tokens),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Pointer {
    Const,
    Mut,
}

impl ToTokens for Pointer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Pointer::*;
        match self {
            Const => quote!(*const).to_tokens(tokens),
            Mut => quote!(*mut).to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
struct Basetype<'a> {
    pointers: Vec::<Pointer>,
    name: &'a str,
}

impl<'a> Basetype<'a> {
    fn new(name: &'a str) -> Self {
        Self {
            pointers: Default::default(),
            name,
        }
    }
    fn push_pointer(&mut self, pointer: Pointer) {
        self.pointers.push(pointer);
    }
    fn set_pointer_from_vkxml(&mut self, ref_type: &Option<vkxml::ReferenceType>, is_const: bool) {
        match ref_type {
            Some(vkxml::ReferenceType::Pointer) => {
                if is_const {
                    self.pointers.push(Pointer::Const);
                }
                else {
                    self.pointers.push(Pointer::Mut);
                }
            }
            Some(vkxml::ReferenceType::PointerToPointer) => {
                if is_const {
                    self.pointers.push(Pointer::Const);
                    self.pointers.push(Pointer::Mut);
                }
                else {
                    self.pointers.push(Pointer::Mut);
                    self.pointers.push(Pointer::Mut);
                }
            }
            Some(vkxml::ReferenceType::PointerToConstPointer) => {
                if is_const {
                    self.pointers.push(Pointer::Const);
                    self.pointers.push(Pointer::Const);
                }
                else {
                    self.pointers.push(Pointer::Mut);
                    self.pointers.push(Pointer::Const);
                }
            }
            None => {},
        }
    }
}

impl ToTokens for Basetype<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let pointers = &self.pointers;
        let name = self.name.as_code();

        quote!(
            #(#pointers)* #name
        ).to_tokens(tokens);
    }
}

// the size of an array is a String in vkxml
#[derive(Copy, Clone)]
struct Size<'a>(&'a str);

impl ToTokens for Size<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;
        let s = self.0.as_code();
        quote!(#s).to_tokens(tokens);
    }
}

#[derive(Clone)]
enum CtypeInner<'a> {
    Basetype(Basetype<'a>),
    Array(Basetype<'a>, Size<'a>),
}

impl<'a> CtypeInner<'a> {
    fn to_array(self, size: &'a str) -> Self {
        use CtypeInner::*;
        match self {
            Basetype(bt) => CtypeInner::Array(bt, Size(size)),
            Array(bt, _) => CtypeInner::Array(bt, Size(size)),
        }
    }
    fn push_pointer(&mut self, pointer: Pointer) {
        use CtypeInner::*;
        match self {
            Basetype(ref mut bt) => bt.push_pointer(pointer),
            Array(ref mut bt, _) => bt.push_pointer(pointer),
        }
    }
    fn set_pointer_from_vkxml(&mut self, ref_type: &Option<vkxml::ReferenceType>, is_const: bool) {
        use CtypeInner::*;
        match self {
            Basetype(ref mut bt) => bt.set_pointer_from_vkxml(ref_type, is_const),
            Array(ref mut bt, _) => bt.set_pointer_from_vkxml(ref_type, is_const),
        }
    }
}

impl ToTokens for CtypeInner<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use CtypeInner::*;
        match self {
            Basetype(bt) => quote!( #bt ).to_tokens(tokens),
            Array(bt,size) => quote!( [ #bt ; #size ] ).to_tokens(tokens),
        }
    }
}

pub struct Ctype<'a> {
    inner: CtypeInner<'a>,
}

impl<'a> Ctype<'a> {
    pub fn new(basetype: &'a str) -> Self {
        Self {
            inner: CtypeInner::Basetype(Basetype::new(basetype)),
        }
    }
    pub fn set_array(&mut self, size: &'a str) {
        self.inner = self.inner.clone().to_array(size);
    }
    pub fn push_pointer(&mut self, pointer: Pointer) {
        self.inner.push_pointer(pointer);
    }
    pub fn set_pointer_from_vkxml(&mut self, ref_type: &Option<vkxml::ReferenceType>, is_const: bool) {
        self.inner.set_pointer_from_vkxml(ref_type, is_const);
    }
}

impl ToTokens for Ctype<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = &self.inner;
        quote!( #inner ).to_tokens(tokens);
    }
}

pub enum ReturnType<'a> {
    None,
    Some(Ctype<'a>),
}

impl Default for ReturnType<'_> {
    fn default() -> Self {
        ReturnType::None
    }
}

impl<'a> From<Ctype<'a>> for ReturnType<'a> {
    fn from(ct: Ctype<'a>) -> Self {
        ReturnType::Some(ct)
    }
}

impl ToTokens for ReturnType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ReturnType::None => quote!( () ).to_tokens(tokens),
            ReturnType::Some(ct) => quote!( #ct ).to_tokens(tokens),
        }
    }
}

pub struct Cfield<'a> {
    vis: Visability,
    name: &'a str,
    ty: Ctype<'a>,
}

impl<'a> Cfield<'a> {
    pub fn new(name: &'a str, ty: Ctype<'a>) -> Self {
        Self {
            vis: Default::default(),
            name,
            ty,
        }
    }

    pub fn set_public(&mut self) {
        self.vis = Visability::Public;
    }
}

impl ToTokens for Cfield<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let vis = &self.vis;
        let name = case::camel_to_snake(self.name).as_code();
        let ty = &self.ty;

        quote!( #vis #name : #ty ).to_tokens(tokens);
    }
}