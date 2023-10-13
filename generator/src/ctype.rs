use std::borrow::Borrow;

use krs_quote::krs_quote_with;

use crate::utils::{self, case};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Visibility {
    Private,
    Public,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Private
    }
}

impl krs_quote::ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use Visibility::*;
        match self {
            Private => {}
            Public => krs_quote_with!(tokens <- pub ),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Pointer {
    Const,
    Mut,
}

impl krs_quote::ToTokens for Pointer {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use Pointer::*;
        match self {
            Const => krs_quote_with!(tokens <- *const ),
            Mut => krs_quote_with!(tokens <- *mut ),
        }
    }
}

#[derive(Clone, Debug)]
struct Basetype {
    pointers: Vec<Pointer>,
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
}

impl krs_quote::ToTokens for Basetype {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let pointers = &self.pointers;
        let name = self.name;

        krs_quote_with!( tokens <-
            {@* {@pointers}} {@name}
        );
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

impl krs_quote::ToTokens for Size {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let s = self.0;
        krs_quote_with!(tokens <- {@s} );
    }
}

#[derive(Clone, Debug)]
struct CtypeInner {
    basetype: Basetype,
    array: Vec<Size>,
}

impl CtypeInner {
    fn push_array(&mut self, size: impl Into<utils::VkTyName>) {
        let size = size.into();
        self.array.push(Size(size));
    }
    fn push_pointer(&mut self, pointer: Pointer) {
        self.basetype.push_pointer(pointer);
    }
}

impl krs_quote::ToTokens for CtypeInner {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let bt = &self.basetype;
        let array = &self.array;

        if array.len() > 1 {
            panic!("handling of multidimensional arrays is not good right now");
        }

        if let Some(size) = array.iter().next() {
            krs_quote_with!(tokens <- [ {@bt} ; {@size}] );
        } else {
            krs_quote_with!(tokens <- {@bt} );
        }
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
    external: bool,
}

impl Ctype {
    pub fn new(basetype: impl Into<utils::VkTyName>) -> Self {
        Self {
            inner: CtypeInner {
                basetype: Basetype::new(basetype),
                array: Default::default(),
            },
            bit_width: Default::default(),
            external: false,
        }
    }
    pub fn push_array(&mut self, size: impl Into<utils::VkTyName>) {
        self.inner.push_array(size);
    }
    pub fn push_pointer(&mut self, pointer: Pointer) {
        self.inner.push_pointer(pointer);
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
    pub fn set_external(&mut self) {
        self.external = true;
    }
    pub fn is_external(&self) -> bool {
        self.external
    }
    pub fn name(&self) -> utils::VkTyName {
        self.inner.basetype.name
    }
}

impl krs_quote::ToTokens for Ctype {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let inner = &self.inner;
        krs_quote_with!(tokens <- {@inner} );
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

impl krs_quote::ToTokens for ReturnType {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        match self {
            ReturnType::None => krs_quote_with!(tokens <- () ),
            ReturnType::Some(ct) => krs_quote_with!(tokens <- {@ct} ),
        }
    }
}

#[derive(Clone)]
pub struct Cfield {
    vis: Visibility,
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
        self.vis = Visibility::Public;
    }
}

impl krs_quote::ToTokens for Cfield {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use crate::utils::StrAsCode;

        let vis = &self.vis;
        let name = case::camel_to_snake(self.name.borrow()).as_code();
        let ty = &self.ty;

        krs_quote_with!(tokens <-
            {@vis} {@name} : {@ty}
        );
    }
}
