use std::borrow::{Borrow, Cow};

use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

use crate::utils::{self, case};

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, Debug)]
struct Basetype {
    pointers: Vec::<Pointer>,
    name: utils::VkTyName,
}

impl Basetype {
    fn new(name: impl Into<utils::VkTyName>) -> Self {
        let name = name.into();
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

impl ToTokens for Basetype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let pointers = &self.pointers;
        let name = self.name;

        quote!(
            #(#pointers)* #name
        ).to_tokens(tokens);
    }
}

impl PartialEq for Basetype {
    fn eq(&self, other: &Self) -> bool {
        for (me, other) in self.pointers.iter().zip(other.pointers.iter()) {
            if me != other {
                return false;
            }
        }
        self.name == other.name
    }
}

impl Eq for Basetype {}

// the size of an array is a String in vkxml
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Size(utils::VkTyName);

impl ToTokens for Size {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;
        let s = self.0;
        quote!(#s).to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
struct CtypeInner {
    basetype: Basetype,
    array: Vec<Size>
}

impl CtypeInner {
    fn push_array(&mut self, size: impl Into<utils::VkTyName>) {
        let size = size.into();
        self.array.push(Size(size));
    }
    fn push_pointer(&mut self, pointer: Pointer) {
        self.basetype.push_pointer(pointer);
    }
    fn set_pointer_from_vkxml(&mut self, ref_type: &Option<vkxml::ReferenceType>, is_const: bool) {
        self.basetype.set_pointer_from_vkxml(ref_type, is_const);
    }
}

impl ToTokens for CtypeInner {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let bt = &self.basetype;
        let array = &self.array;

        let mut accumulate = quote!(#bt);
        for size in array {
            accumulate = quote!( [ #accumulate ; #size] );
        }

        accumulate.to_tokens(tokens);
    }
}

impl PartialEq for CtypeInner {
    fn eq(&self, other: &Self) -> bool {
        for (me, other) in self.array.iter().zip(other.array.iter()) {
            if me != other {
                return false;
            }
        }
        self.basetype == other.basetype
    }
}

impl Eq for CtypeInner {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ctype {
    inner: CtypeInner,
    bit_width: Option<u8>,
}

impl Ctype {
    pub fn new(basetype: impl Into<utils::VkTyName>) -> Self {
        Self {
            inner: CtypeInner {
                basetype: Basetype::new(basetype),
                array: Default::default(),
            },
            bit_width: Default::default(),
        }
    }
    pub fn push_array(&mut self, size: impl Into<utils::VkTyName>) {
        self.inner.push_array(size);
    }
    pub fn push_pointer(&mut self, pointer: Pointer) {
        self.inner.push_pointer(pointer);
    }
    pub fn set_pointer_from_vkxml(&mut self, ref_type: &Option<vkxml::ReferenceType>, is_const: bool) {
        self.inner.set_pointer_from_vkxml(ref_type, is_const);
    }
    pub fn set_bit_width(&mut self, bit_width: u8) {
        self.bit_width = Some(bit_width);
    }
    pub fn basetype(&self) -> &str {
        &self.inner.basetype.name
    }
    pub fn bit_width(&self) -> Option<u8> {
        self.bit_width
    }
    pub fn is_array(&self) -> bool {
        self.inner.array.len() > 0
    }
    pub fn is_pointer(&self) -> bool {
        self.inner.basetype.pointers.len() > 0
    }
}

impl ToTokens for Ctype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = &self.inner;
        quote!( #inner ).to_tokens(tokens);
    }
}

pub enum ReturnType {
    None,
    Some(Ctype),
}

impl Default for ReturnType {
    fn default() -> Self {
        ReturnType::None
    }
}

impl From<Ctype> for ReturnType {
    fn from(ct: Ctype) -> Self {
        ReturnType::Some(ct)
    }
}

impl ToTokens for ReturnType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ReturnType::None => quote!( () ).to_tokens(tokens),
            ReturnType::Some(ct) => quote!( #ct ).to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
pub struct Cfield {
    vis: Visability,
    pub name: utils::VkTyName,
    pub ty: Ctype,
}

impl Cfield {
    pub fn new(name: impl Into<utils::VkTyName>, ty: Ctype) -> Self {
        let name = name.into();
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

impl ToTokens for Cfield {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let vis = &self.vis;
        let name = case::camel_to_snake(self.name.borrow()).as_code();
        let ty = &self.ty;

        quote!( #vis #name : #ty ).to_tokens(tokens);
    }
}