use std::marker::PhantomData;

use krs_quote::krs_quote_with;

use crate::utils::{VkTyName, VecMap, case};

use crate::ctype;

// =================================================================
/// TypeDef
/// for defining Vulkan type aliases
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: VkTyName,
    pub ty: VkTyName,
}

impl TypeDef {
    pub fn new(name: impl Into<VkTyName>, ty: impl Into<VkTyName>) -> Self {
        let name = name.into();
        let ty = ty.into();
        Self {
            name,
            ty,
        }
    }
}

impl krs_quote::ToTokens for TypeDef {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let ty = self.ty;
        krs_quote_with!(tokens <-
            pub type {@name} = {@ty};
        );
    }
}

// =================================================================
/// Bitmask
/// for defining Vulkan Flags types
pub struct Bitmask {
    name: VkTyName,
    ty: VkTyName,
}

impl Bitmask {
    pub fn new(name: impl Into<VkTyName>, ty: impl Into<VkTyName>) -> Self {
        let name = name.into();
        let ty = ty.into();
        Self {
            name,
            ty,
        }
    }
}

impl krs_quote::ToTokens for Bitmask {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let ty = self.ty;
        krs_quote_with!(tokens <-
            #[repr(transparent)]
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct {@name}(pub(crate) {@ty});
            impl VkBitmaskType for {@name} {
                /// the underling type of the flags e.g. VkFlags (uint32_t) or VkFlags64 (uint64_t)
                type RawType = {@ty};
                // type Verifier: VerifyBits;
                fn from_bit_type_list<L: BitList<Self::RawType, Self>>(_: L) -> Self {
                    Self(L::FLAGS)
                }
            }
            vk_bitflags_wrapped!({@name}, {@ty});
        );
    }
}

// =================================================================
/// Struct
/// for defining Vulkan struct types
pub struct Struct2 {
    name: VkTyName,
    fields: Vec<ctype::Cfield>,
    pub non_normative: bool,
}

impl Struct2 {
    pub fn new(name: impl Into<VkTyName>) -> Self {
        let name = name.into();
        Self {
            name,
            fields: Default::default(),
            non_normative: false,
        }
    }
    pub fn push_field(&mut self, field: ctype::Cfield) {
        self.fields.push(field);
    }
    pub fn non_normative(&mut self) {
        self.non_normative = true;
    }
}

impl krs_quote::ToTokens for Struct2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;

        let generics = self.fields.iter().filter_map(|field| {
            if field.ty.is_external() {
                Some(field.ty.name())
            }
            else {
                None
            }
        });

        match self.non_normative {
            false => {
                let fields = &self.fields;
                krs_quote_with!(tokens <-
                    #[repr(C)]
                    #[derive(Copy, Clone, Debug)]
                    pub struct {@name} <{@,* {@generics}}> {
                        {@* {@fields} , }
                    }
                );
            }
            true => {
                let fields = BitFieldIter::new(self.fields.iter());
                krs_quote_with!(tokens <-
                    #[repr(C)]
                    #[repr(packed)]
                    #[derive(Copy, Clone, Debug)]
                    pub struct {@name} <{@,* {@generics}}> {
                        {@* {@fields} , }
                    }
                );
            }
        }
    }
}

// in C, bitfields should be compiled to fit into the same space
// this iterates over potential bitfields and emits one field for all bit fields that should fit within the one field
// we assume that the vulkan spec only uses bit fields efficiently and tightly packs and uses all space
#[derive(Clone)]
struct BitFieldIter<'a, I: Iterator<Item=&'a ctype::Cfield>> {
    fields: I,
    _p: PhantomData<&'a I::Item>,
}

impl<'a, I: Iterator<Item=&'a ctype::Cfield>> BitFieldIter<'a, I> {
    fn new(i: impl IntoIterator<IntoIter=I>) -> Self {
        Self {
            fields: i.into_iter(),
            _p: PhantomData,
        }
    }
}

impl<'a, I: Iterator<Item=&'a ctype::Cfield>> Iterator for BitFieldIter<'a, I> {
    type Item = ctype::Cfield;
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

            // check how many bit_fields fit within one field
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
pub struct Union {
    name: VkTyName,
    fields: Vec<ctype::Cfield>,
}

impl Union {
    pub fn new(name: impl Into<VkTyName>) -> Self {
        let name = name.into();
        Self {
            name,
            fields: Default::default(),
        }
    }
    pub fn extend_fields(&mut self, fields: impl IntoIterator<Item=ctype::Cfield>) {
        self.fields.extend(fields);
    }
}

impl krs_quote::ToTokens for Union {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name;
        let fields = &self.fields;
        let field_names = fields.iter().map(|field| case::camel_to_snake(field.name.as_ref()).as_code());

        krs_quote_with!(tokens <-
            #[repr(C)]
            #[derive(Copy, Clone)]
            pub union {@name} {
                {@* {@fields} , }
            }
            impl std::fmt::Debug for {@name} {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    unsafe {
                        f.debug_struct(stringify!({@name}))
                            {@* .field(stringify!({@field_names}), &self.{@field_names})}
                            .finish()
                    }
                }
            }
        );
    }
}

// =================================================================
/// Handle
/// for defining Vulkan Handle types
pub struct Handle2 {
    name: VkTyName,
    dispatch: bool,
}

impl Handle2 {
    pub fn new(name: impl Into<VkTyName>, dispatch: bool) -> Self {
        let name = name.into();
        Self {
            name,
            dispatch,
        }
    }
}

impl krs_quote::ToTokens for Handle2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let ty = match self.dispatch {
            true => {
                let mut ty = ctype::Ctype::new("c_void");
                ty.push_pointer(ctype::Pointer::Const);
                ty
            }
            false => ctype::Ctype::new("u64"),
        };

        krs_quote_with!(tokens <-
            #[repr(transparent)]
            #[derive(Copy, Clone)]
            pub struct {@name} {
                pub handle: {@ty},
            }
            impl ::std::fmt::Debug for {@name} {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    write!(f, concat!(stringify!({@name}), "({:?})"), self.handle)
                }
            }
        );
    }
}

// =================================================================
/// Enumerations
/// for defining Vulkan enum types
/// we represent Vulkan C enums as rust structs, and the variants will be associated constants
/// should skip generating this for FlagBits definitions since we will define the actual bits
/// as associated constants on the actual Bitmask type
pub struct Enum2 {
    name: VkTyName,
}

impl Enum2 {
    pub fn new(name: impl Into<VkTyName>) -> Self {
        let name = name.into();
        Self {
            name
        }
    }
}

impl krs_quote::ToTokens for Enum2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        krs_quote_with!(tokens <-
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            pub struct {@name}(pub(crate) i32);
        );
    }
}

// =================================================================
/// Function Pointers
/// for defining Vulkan function pointer types
pub struct FunctionPointer {
    pub name: VkTyName,
    fields: Vec<ctype::Cfield>,
    return_type: ctype::ReturnType,
}

impl FunctionPointer {
    pub fn new(name: impl Into<VkTyName>) -> Self {
        let name = name.into();
        Self {
            name,
            fields: Default::default(),
            return_type: Default::default(),
        }
    }
    pub fn extend_fields(&mut self, fields: impl IntoIterator<Item=ctype::Cfield>) {
        self.fields.extend(fields);
    }
    pub fn set_return_type(&mut self, return_type: impl Into<ctype::ReturnType>) {
        self.return_type = return_type.into();
    }
}

impl krs_quote::ToTokens for FunctionPointer {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name;
        let fn_type = format!("FptrTy{}", self.name).as_code();

        let fields = &self.fields;
        let return_type = &self.return_type;

        let generics: Vec<_> = self.fields.iter().filter_map(|field|{
            if field.ty.is_external() {
                Some(field.ty.name())
            }
            else {
                None
            }
        }).collect();

        let unsafe_get = if generics.len() == 0 {
            None
        }
        else {
            Some( krs_quote::Token::from("unsafe") )
        };

        krs_quote_with!(tokens <-
            #[allow(non_camel_case_types)]
            pub type {@fn_type} <{@,* {@generics}}> = unsafe extern "system" fn(
                {@,* {@fields} }
            ) -> {@return_type};

            #[repr(transparent)]
            #[derive(Copy, Clone)]
            pub struct {@name}(PFN_vkVoidFunction);

            impl {@name} {
                pub unsafe fn new(fptr: PFN_vkVoidFunction) -> Self {
                    Self(fptr)
                }
                pub {@unsafe_get} fn get_fptr<{@,* {@generics}}>(self) -> {@fn_type}<{@,* {@generics}}> {
                    unsafe { std::mem::transmute(self) }
                }
            }

            impl std::fmt::Debug for {@name} {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", stringify!({@name}))
                }
            }
        );
    }
}

// =================================================================
/// Definitions
/// collect all definitions together for outputting together
#[derive(Default)]
pub struct Definitions2 {
    pub type_defs: Vec<TypeDef>,
    pub bitmasks: Vec<Bitmask>,
    pub structs: VecMap<VkTyName, Struct2>,
    pub unions: Vec<Union>,
    pub handles: Vec<Handle2>,
    pub enumerations: Vec<Enum2>,
    pub function_pointers: Vec<FunctionPointer>,
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

impl krs_quote::ToTokens for Definitions2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let type_defs = &self.type_defs;
        let bitmasks = &self.bitmasks;
        let structs = self.structs.iter();
        let unions = &self.unions;
        let handles = &self.handles;
        let enumerations = &self.enumerations;
        let function_pointers = &self.function_pointers;

        krs_quote_with!(tokens <-
            {@* {@type_defs} }
            {@* {@bitmasks} }
            {@* {@structs} }
            {@* {@unions} }
            {@* {@handles} }
            {@* {@enumerations} }
            {@* {@function_pointers} }
        );
    }
}