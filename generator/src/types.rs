use std::collections::HashSet;
use std::marker::PhantomData;

use krs_quote::{krs_quote_with, to_tokens_closure, ToTokens};

use crate::utils::{case, VecMap, VkTyName};

use crate::ctype;

// =================================================================
/// TypeDef
/// for defining Vulkan type aliases
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: VkTyName,
    pub ty: VkTyName,
    pub ptr: bool,
}

impl TypeDef {
    pub fn new(name: impl Into<VkTyName>, ty: impl Into<VkTyName>) -> Self {
        let name = name.into();
        let ty = ty.into();
        Self {
            name,
            ty,
            ptr: false,
        }
    }
    pub fn set_ptr(&mut self) {
        self.ptr = true;
    }
}

impl krs_quote::ToTokens for TypeDef {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let ty = self.ty;
        match self.ptr {
            true => {
                krs_quote_with!(tokens <-
                    pub type {@name} = *mut {@ty};
                );
            }
            false => {
                krs_quote_with!(tokens <-
                    pub type {@name} = {@ty};
                );
            }
        }
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
        Self { name, ty }
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
            unsafe impl crate::flag_traits::FlagType for {@name} {
                const EMPTY: Self = Self(0);
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

struct StructToToken<'a> {
    s: &'a Struct2,
    g: &'a HashSet<VkTyName>,
}

impl krs_quote::ToTokens for StructToToken<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.s.name;

        let generics = self.s.fields.iter().filter_map(|field| {
            if field.ty.is_external() || self.g.contains(&field.ty.name()) {
                Some(field.ty.name())
            } else {
                None
            }
        });

        match self.s.non_normative {
            false => {
                let fields = &self.s.fields;
                krs_quote_with!(tokens <-
                    #[repr(C)]
                    #[derive(Copy, Clone, Debug)]
                    pub struct {@name} <{@,* {@generics}}> {
                        {@* {@fields} , }
                    }
                );
            }
            true => {
                let fields = BitFieldIter::new(self.s.fields.iter());
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
struct BitFieldIter<'a, I: Iterator<Item = &'a ctype::Cfield>> {
    fields: I,
    _p: PhantomData<&'a I::Item>,
}

impl<'a, I: Iterator<Item = &'a ctype::Cfield>> BitFieldIter<'a, I> {
    fn new(i: impl IntoIterator<IntoIter = I>) -> Self {
        Self {
            fields: i.into_iter(),
            _p: PhantomData,
        }
    }
}

impl<'a, I: Iterator<Item = &'a ctype::Cfield>> Iterator for BitFieldIter<'a, I> {
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
            let mut name = field.name.to_string();
            while bits < field_bit_size {
                let next_field = self.fields.next().expect("error: expected another field");
                // assert_eq!(basetype, next_field.ty.basetype(), "error: expect that neighbor bitfields have same basetype");
                assert_eq!(
                    next_field.ty.is_array(),
                    false,
                    "error: expect non array type for bit fields"
                );
                assert_eq!(
                    next_field.ty.is_pointer(),
                    false,
                    "error: expect non pointer type for bit fields"
                );
                bits += next_field
                    .ty
                    .bit_width()
                    .expect("error: expected next field to have bit_width");
                name = format!("{}_and_{}", name, next_field.name);
            }

            assert_eq!(
                bits, field_bit_size,
                "error: expected total bits to be equal to field bit size"
            );

            let ty = field.ty.clone();
            let field = ctype::Cfield::new(name, ty);

            Some(field)
        } else {
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
    pub fn extend_fields(&mut self, fields: impl IntoIterator<Item = ctype::Cfield>) {
        self.fields.extend(fields);
    }
}

impl krs_quote::ToTokens for Union {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use crate::utils::StrAsCode;

        let name = self.name;
        let fields = &self.fields;
        let field_names = fields
            .iter()
            .map(|field| case::camel_to_snake(field.name.as_ref()).as_code());

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
        Self { name, dispatch }
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
        Self { name }
    }
}

impl krs_quote::ToTokens for Enum2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        krs_quote_with!(tokens <-
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[repr(transparent)]
            pub struct {@name}(pub(crate) i32);
            impl {@name} {
                pub const fn is(self, other: Self) -> bool {
                    self.0 == other.0
                }
            }
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
    pub fn extend_fields(&mut self, fields: impl IntoIterator<Item = ctype::Cfield>) {
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

        let generics: Vec<_> = self
            .fields
            .iter()
            .filter_map(|field| {
                if field.ty.is_external() {
                    Some(field.ty.name())
                } else {
                    None
                }
            })
            .collect();

        let unsafe_get = if generics.len() == 0 {
            None
        } else {
            Some(krs_quote::Token::from("unsafe"))
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

#[derive(Clone, Copy)]
enum TypeIndex {
    TypeDef(usize),
    Bitmask(usize),
    Struct(usize),
    Union(usize),
    Handle(usize),
    Enum(usize),
    FunctionPointer(usize),
}

// =================================================================
/// Definitions
/// collect all definitions together for outputting together
#[derive(Default)]
pub struct Types {
    map: VecMap<VkTyName, TypeIndex>,

    type_defs: Vec<Type<TypeDef>>,
    bitmasks: Vec<Type<Bitmask>>,
    structs: Vec<Type<Struct2>>,
    unions: Vec<Type<Union>>,
    handles: Vec<Type<Handle2>>,
    enumerations: Vec<Type<Enum2>>,
    function_pointers: Vec<Type<FunctionPointer>>,

    // in order to avoid external ".h" files and c libraries, we do not generate the external types and just treat them generically
    // to achieve this, we treat such types as generic, and a user needs to determine the correct type
    generic_types: HashSet<VkTyName>,
}

struct Type<T> {
    enabled: bool,
    ty: T,
}

impl<T> Type<T> {
    fn new(ty: T) -> Self {
        // *********** TODO ******************
        // should be disabled by default
        Self { enabled: true, ty }
    }
}

impl<T: ToTokens> ToTokens for Type<T> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        if self.enabled {
            self.ty.to_tokens(tokens);
        }
    }
}

impl Types {
    // ************ Generic types *********************
    pub fn is_generic(&self, name: VkTyName) -> bool {
        self.generic_types.contains(&name)
    }

    pub fn add_generic_type(&mut self, name: VkTyName) {
        self.generic_types.insert(name);
    }

    // ************ TypeDef *********************
    pub fn insert_type_def(&mut self, type_def: TypeDef) {
        let index = self.type_defs.len();
        let name = type_def.name;
        self.type_defs.push(Type::new(type_def));
        self.map.push(name, TypeIndex::TypeDef(index));
    }

    pub fn type_defs_to_tokens(&self) -> impl ToTokens + use<'_> {
        to_tokens_closure!(tokens {
            for type_def in self.type_defs.iter() {
                type_def.to_tokens(tokens);
            }
        })
    }

    // ************ Bitmask *********************
    pub fn insert_bitmask(&mut self, bitmask: Bitmask) {
        let index = self.bitmasks.len();
        let name = bitmask.name;
        self.bitmasks.push(Type::new(bitmask));
        self.map.push(name, TypeIndex::Bitmask(index));
    }

    pub fn bitmasks_to_tokens(&self) -> impl ToTokens + use<'_> {
        to_tokens_closure!(tokens {
            for bitmask in self.bitmasks.iter() {
                bitmask.to_tokens(tokens);
            }
        })
    }

    // ************ Struct *********************
    pub fn insert_struct(&mut self, stct: Struct2) {
        let index = self.structs.len();
        let name = stct.name;
        self.structs.push(Type::new(stct));
        self.map.push(name, TypeIndex::Struct(index));
    }

    pub fn structs_to_tokens(&self) -> impl ToTokens + use<'_> {
        to_tokens_closure!(tokens {
            for s in self.structs.iter() {
                StructToToken {
                    s: &s.ty,
                    g: &self.generic_types,
                }
                .to_tokens(tokens)
            }
        })
    }

    // ************ Union *********************
    pub fn insert_union(&mut self, u: Union) {
        let index = self.unions.len();
        let name = u.name;
        self.unions.push(Type::new(u));
        self.map.push(name, TypeIndex::Union(index));
    }

    pub fn unions_to_tokens(&self) -> impl ToTokens + use<'_> {
        to_tokens_closure!(tokens {
            for u in self.unions.iter() {
                u.to_tokens(tokens);
            }
        })
    }

    // ************ Handle *********************
    pub fn insert_handle(&mut self, handle: Handle2) {
        let index = self.handles.len();
        let name = handle.name;
        self.handles.push(Type::new(handle));
        self.map.push(name, TypeIndex::Handle(index));
    }

    pub fn handles_to_tokens(&self) -> impl ToTokens + use<'_> {
        to_tokens_closure!(tokens {
            for handle in self.handles.iter() {
                handle.to_tokens(tokens);
            }
        })
    }

    // ************ Enum *********************
    pub fn insert_enum(&mut self, enm: Enum2) {
        let index = self.enumerations.len();
        let name = enm.name;
        self.enumerations.push(Type::new(enm));
        self.map.push(name, TypeIndex::Enum(index));
    }

    pub fn enums_to_tokens(&self) -> impl ToTokens + use<'_> {
        to_tokens_closure!(tokens {
            for enm in self.enumerations.iter() {
                enm.to_tokens(tokens);
            }
        })
    }

    // ************ FunctionPointer *********************
    pub fn insert_function_pointer(&mut self, fptr: FunctionPointer) {
        let index = self.function_pointers.len();
        let name = fptr.name;
        self.function_pointers.push(Type::new(fptr));
        self.map.push(name, TypeIndex::FunctionPointer(index));
    }

    pub fn function_pointers_to_tokens(&self) -> impl ToTokens + use<'_> {
        to_tokens_closure!(tokens {
            for fptr in self.function_pointers.iter() {
                fptr.to_tokens(tokens);
            }
        })
    }
}