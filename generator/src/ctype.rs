
use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

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
enum Pointer {
    None,
    Const,
    Mut,
    ConstToConst,
    ConstToMut,
    MutToConst,
    MutToMut,
}

impl Default for Pointer {
    fn default() -> Self {
        Pointer::None
    }
}

impl ToTokens for Pointer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Pointer::*;
        match self {
            None => {}
            Const => quote!(*const).to_tokens(tokens),
            Mut => quote!(*mut).to_tokens(tokens),
            ConstToConst => quote!(*const *const).to_tokens(tokens),
            ConstToMut => quote!(*const *mut).to_tokens(tokens),
            MutToConst => quote!(*mut *const).to_tokens(tokens),
            MutToMut => quote!(*mut *mut).to_tokens(tokens),
        }
    }
}

#[derive(Copy, Clone)]
struct Basetype<'a> {
    pointer: Pointer,
    name: &'a str,
}

impl<'a> Basetype<'a> {
    fn new(name: &'a str) -> Self {
        Self {
            pointer: Default::default(),
            name,
        }
    }
    fn set_pointer_from_vkxml(&mut self, ref_type: vkxml::ReferenceType, is_const: bool) {
        match ref_type {
            vkxml::ReferenceType::Pointer => {
                if is_const {
                    self.pointer = Pointer::Const;
                }
                else {
                    self.pointer = Pointer::Mut;
                }
            }
            vkxml::ReferenceType::PointerToPointer => {
                if is_const {
                    self.pointer = Pointer::ConstToMut;
                }
                else {
                    self.pointer = Pointer::MutToMut;
                }
            }
            vkxml::ReferenceType::PointerToConstPointer => {
                if is_const {
                    self.pointer = Pointer::ConstToConst;
                }
                else {
                    self.pointer = Pointer::MutToConst;
                }
            }
        }
    }
}

impl ToTokens for Basetype<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let pointer = self.pointer;
        let name = self.name.as_code();

        quote!(
            #pointer #name
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

#[derive(Copy, Clone)]
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
    fn set_pointer_from_vkxml(&mut self, ref_type: vkxml::ReferenceType, is_const: bool) {
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
    vis: Visability,
    inner: CtypeInner<'a>,
}

impl<'a> Ctype<'a> {
    pub fn new(basetype: &'a str) -> Self {
        Self {
            vis: Default::default(),
            inner: CtypeInner::Basetype(Basetype::new(basetype)),
        }
    }
    pub fn set_public(&mut self) {
        self.vis = Visability::Public;
    }
    pub fn set_array(&mut self, size: &'a str) {
        self.inner = self.inner.to_array(size);
    }
    fn set_pointer_from_vkxml(&mut self, ref_type: vkxml::ReferenceType, is_const: bool) {
        self.inner.set_pointer_from_vkxml(ref_type, is_const);
    }
}

impl ToTokens for Ctype<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = self.vis;
        let inner = &self.inner;
        quote!( #vis #inner ).to_tokens(tokens);
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
