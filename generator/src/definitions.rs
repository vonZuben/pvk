
use std::marker::PhantomData;

use quote::{quote, ToTokens};

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

use crate::utils;

use crate::ctype;

// =================================================================
/// TypeDef
/// for defining Vulkan type aliases
#[derive(Debug, Clone)]
pub struct TypeDef<'a> {
    pub name: &'a str,
    pub ty: &'a str,
}

impl<'a> TypeDef<'a> {
    pub fn new(name: &'a str, ty: &'a str) -> Self {
        Self {
            name,
            ty,
        }
    }
}

impl ToTokens for TypeDef<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_code();
        let ty = self.ty.as_code();
        quote!( pub type #name = #ty; ).to_tokens(tokens);
    }
}

// =================================================================
/// Bitmask
/// for defining Vulkan Flags types
pub struct Bitmask<'a> {
    name: &'a str,
    ty: &'a str,
}

impl<'a> Bitmask<'a> {
    pub fn new(name: &'a str, ty: &'a str) -> Self {
        Self {
            name,
            ty,
        }
    }
}

impl ToTokens for Bitmask<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_code();
        let ty = self.ty.as_code();
        quote!(
            #[repr(transparent)]
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct #name(pub(crate) #ty);
            vk_bitflags_wrapped!(#name);
        ).to_tokens(tokens);
    }
}

// =================================================================
/// Struct
/// for defining Vulkan struct types
pub struct Struct2<'a> {
    name: &'a str,
    fields: Vec<ctype::Cfield<'a>>,
    pub non_normative: bool,
}

impl<'a> Struct2<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            fields: Default::default(),
            non_normative: false,
        }
    }
    pub fn extend_fields(&mut self, fields: impl IntoIterator<Item=ctype::Cfield<'a>>) {
        self.fields.extend(fields);
    }
    pub fn push_field(&mut self, field: ctype::Cfield<'a>) {
        self.fields.push(field);
    }
    pub fn non_normative(&mut self) {
        self.non_normative = true;
    }
}

impl ToTokens for Struct2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name.as_code();
        
        match self.non_normative {
            false => {
                let fields = &self.fields;
                quote!(
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    pub struct #name {
                        #( #fields , )*
                    }
                ).to_tokens(tokens);
            }
            true => {
                let fields = BitFieldIter::new(self.fields.iter());
                quote!(
                    #[repr(C)]
                    #[repr(packed)]
                    #[derive(Copy, Clone)]
                    pub struct #name {
                        #( #fields , )*
                    }
                ).to_tokens(tokens);
            }
        }        
    }
}

// in C, bitfields should be compiled to fit into the same space
// this iterates over potential bitfields and emits one field for all bit fields that should fit within the one field
// we assume that the vulkan spec only uses bit fields efficiently and tightly packs and uses all space
struct BitFieldIter<'a, I: Iterator<Item=&'a ctype::Cfield<'a>>> {
    fields: I,
    _p: PhantomData<&'a I::Item>,
}

impl<'a, I: Iterator<Item=&'a ctype::Cfield<'a>>> BitFieldIter<'a, I> {
    fn new(i: impl IntoIterator<IntoIter=I>) -> Self {
        Self {
            fields: i.into_iter(),
            _p: PhantomData,
        }
    }
}

impl<'a, I: Iterator<Item=&'a ctype::Cfield<'a>>> Iterator for BitFieldIter<'a, I> {
    type Item = ctype::Cfield<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let field = self.fields.next()?;

        // check if field is a bit_field
        if let Some(mut bits) = field.ty.bit_width() {
            // determine size of field
            let basetype = field.ty.basetype();
            let field_bit_size = match basetype {
                x if x.contains("8") => 8,
                x if x.contains("16") => 16,
                x if x.contains("32") => 32,
                x if x.contains("64") => 64,
                _ => panic!("error: unknown field_bit_size"),
            };

            // check how many bit_fields fit wihtin one field
            let mut name =  field.name.to_string();
            while bits < field_bit_size {
                let next_field = self.fields.next().expect("error: expected another field");
                // assert_eq!(basetype, next_field.ty.basetype(), "error: expect that neighbor bitfields have same basetype");
                assert_eq!(next_field.ty.is_array(), false, "error: expect non array type for bit fields");
                assert_eq!(next_field.ty.is_pointer(), false, "error: expect non pointer type for bit fields");
                bits += next_field.ty.bit_width().expect("error: expected next field to have bit_width");
                name = format!("{}_and_{}", name, next_field.name);
            }

            assert_eq!(bits, field_bit_size, "error: expected total bits to be equal to field bit size");

            let ty = field.ty.clone();
            let field = ctype::Cfield::new(name, ty);

            Some(field)
        }
        else {
            Some(field.clone())
        }
    }
}

// =================================================================
/// Union
/// for defining Vulkan union types
pub struct Union<'a> {
    name: &'a str,
    fields: Vec<ctype::Cfield<'a>>,
}

impl<'a> Union<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            fields: Default::default(),
        }
    }
    pub fn extend_fields(&mut self, fields: impl IntoIterator<Item=ctype::Cfield<'a>>) {
        self.fields.extend(fields);
    }
}

impl ToTokens for Union<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name.as_code();
        let fields = &self.fields;

        quote!(
            #[repr(C)]
            #[derive(Copy, Clone)]
            pub union #name {
                #( #fields , )*
            }
        ).to_tokens(tokens);
    }
}

// =================================================================
/// Handle
/// for defining Vulkan Handle types
pub struct Handle2<'a> {
    name: &'a str,
    dispatch: bool,
}

impl<'a> Handle2<'a> {
    pub fn new(name: &'a str, dispatch: bool) -> Self {
        Self {
            name,
            dispatch,
        }
    }
}

impl ToTokens for Handle2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name.as_code();
        let ty = match self.dispatch {
            true => {
                let mut ty = ctype::Ctype::new("c_void");
                ty.push_pointer(ctype::Pointer::Const);
                ty
            }
            false => ctype::Ctype::new("u64"),
        };

        quote!(
            #[repr(transparent)]
            #[derive(Copy, Clone)]
            pub struct #name {
                pub handle: #ty,
            }
            impl ::std::fmt::Debug for #name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    write!(f, concat!(stringify!(#name), "({:?})"), self.handle)
                }
            }
        ).to_tokens(tokens);
    }
}

// =================================================================
/// Enumerations
/// for defining Vulkan enum typs
/// we represent Vulkan C enums as rust structs, and the variants will be associated constants
/// should skip generating this for FlagBits definitions since we will define the actual bits
/// as associated constants on the actual Bitmask type
pub struct Enum2<'a> {
    name: &'a str,
}

impl<'a> Enum2<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name
        }
    }
}

impl ToTokens for Enum2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;
        let name = self.name.as_code();
        quote!(
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            pub struct #name(pub(crate) i32);
        ).to_tokens(tokens);
    }
}


// =================================================================
/// Funtion Pointers
/// for defining Vulkan function pointer types
pub struct FunctionPointer<'a> {
    pub name: &'a str,
    fields: Vec<ctype::Cfield<'a>>,
    return_type: ctype::ReturnType<'a>,
}

impl<'a> FunctionPointer<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            fields: Default::default(),
            return_type: Default::default(),
        }
    }
    pub fn extend_fields(&mut self, fields: impl IntoIterator<Item=ctype::Cfield<'a>>) {
        self.fields.extend(fields);
    }
    pub fn set_return_type(&mut self, return_type: impl Into<ctype::ReturnType<'a>>) {
        self.return_type = return_type.into();
    }
}

impl ToTokens for FunctionPointer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name.as_code();
        let fields = &self.fields;
        let return_type = &self.return_type;

        quote!(
            #[allow(non_camel_case_types)]
            pub type #name = unsafe extern "system" fn(
                #( #fields ),*
            ) -> #return_type;
        ).to_tokens(tokens);
    }
}

// =================================================================
/// Definitions
/// collect all definitions together for outputting together
#[derive(Default)]
pub struct Definitions2<'a> {
    pub type_defs: Vec<TypeDef<'a>>,
    pub bitmasks: Vec<Bitmask<'a>>,
    pub structs: VecMap<&'a str, Struct2<'a>>,
    pub unions: Vec<Union<'a>>,
    pub handles: Vec<Handle2<'a>>,
    pub enumerations: Vec<Enum2<'a>>,
    pub function_pointers: Vec<FunctionPointer<'a>>,
}

//impl<'a> Definitions2<'a> {
//    fn extend_type_defs(&mut self, type_defs: impl IntoIterator<Item=TypeDef<'a>>) {
//        self.type_defs.extend(type_defs);
//    }
//    fn extend_bitmasks(&mut self, bitmasks: impl IntoIterator<Item=Bitmask<'a>>) {
//        self.bitmasks.extend(bitmasks);
//    }
//    fn extend_structs(&mut self, structs: impl IntoIterator<Item=Struct2<'a>>) {
//        self.structs.extend(structs);
//    }
//    fn extend_unions(&mut self, unions: impl IntoIterator<Item=Union<'a>>) {
//        self.unions.extend(unions);
//    }
//    fn extend_handles(&mut self, handles: impl IntoIterator<Item=Handle2<'a>>) {
//        self.handles.extend(handles);
//    }
//    fn extend_function_pointers(&mut self, function_pointers: impl IntoIterator<Item=FunctionPointer<'a>>) {
//        self.function_pointers.extend(function_pointers);
//    }
//}

impl ToTokens for Definitions2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let type_defs = &self.type_defs;
        let bitmasks = &self.bitmasks;
        let structs = self.structs.iter();
        let unions = &self.unions;
        let handles = &self.handles;
        let enumerations = &self.enumerations;
        let function_pointers = &self.function_pointers;

        quote!(
            #( #type_defs )*
            #( #bitmasks )*
            #( #structs )*
            #( #unions )*
            #( #handles )*
            #( #enumerations )*
            #( #function_pointers )*
        ).to_tokens(tokens);
    }
}