
use syn::{Expr};

use quote::quote;
use quote::ToTokens;

use proc_macro2::{TokenStream};

use crate::ty::{self, *};
use crate::global_data;

macro_rules! pipe {

    ( @EXPAND $val:ident => ) => {
        $val
    };

    ( @EXPAND $val:ident => STAGE $f:block $($rest:tt)* ) => {
        {
            let $val = $f;
            let $val = pipe!( @EXPAND $val => $($rest)* );
            $val
        }
    };

    ( @EXPAND $val:ident => STAGE $f:expr ; $($rest:tt)* ) => {
        {
            let $val = $f;
            let $val = pipe!( @EXPAND $val => $($rest)* );
            $val
        }
    };

    ( @EXPAND $val:ident => DONE WHEN $cond:expr => $f:block $($rest:tt)* ) => {
        {
            if $cond {
                $f
            }
            else {
                let $val = pipe!( @EXPAND $val => $($rest)* );
                $val
            }
        }
    };

    ( @EXPAND $val:ident => WHEN $cond:expr => $f:block $($rest:tt)* ) => {
        {
            let $val = if $cond {
                $f
            }
            else {
                $val
            };
            let $val = pipe!( @EXPAND $val => $($rest)* );
            $val
        }
    };

    ( $val:ident => $($stages:tt)+ ) => {
        {
            let $val = pipe!( @EXPAND $val => $($stages)+ );
            $val
        }
    };

    ( $val:ident = $init:expr => $($stages:tt)+ ) => {
        {
            let $val = $init;
            let $val = pipe!( @EXPAND $val => $($stages)+ );
            $val
        }
    };

}

// find the last index of an element matching a condition by searching in reverse
pub trait ReverseIndexFind<T> {
    fn my_rfind(&self, f: impl FnOnce(T) -> bool + Copy) -> Option<usize>;
}

// this is used to find tags by searching for the last lowercase letter and assuming that anything
// after might be a tag since all tags are uppercase suffixes
impl ReverseIndexFind<char> for &'_ str {
    fn my_rfind(&self, f: impl FnOnce(char) -> bool + Copy) -> Option<usize> {
        let mut index = self.len();
        for c in self.chars().rev() {
            if f(c) {
                return Some(index - 1);
            }
            index -= 1;
        }
        None
    }
}

#[macro_export]
macro_rules! variant {
    ( $pattern:path ) => {
        |elem| match elem {
            $pattern(thing) => Some(thing),
            _ => None,
        }
    }
}

pub trait StrAsCode {
    fn as_code(&self) -> TokenStream;
}

// This implementation is intended to convert any string
// into valid tokens
// If you simply want a literal string then don't use this
impl<T> StrAsCode for T where T: AsRef<str> {
    fn as_code(&self) -> TokenStream {
        let rstr = ctype_to_rtype(self.as_ref());
        rstr.parse()
            .expect(format!("error: can't parse {{{}}} as TokenStream", &rstr).as_ref())
    }
}

pub fn structure_type_name<'a>(field: &'a vkxml::Field) -> &'a str {
    let raw_stype = field.type_enums.as_ref().expect("error: sType with no provided value, or not sType field");
    &raw_stype[18..] // cut off the "VK_STRUCTURE_TYPE_" from the begining
}

pub fn field_name_expected(field: &vkxml::Field) -> &str {
    field.name.as_ref().expect("error: field does not have name when expected").as_str()
}

pub fn make_handle_owner_name(name: &str) -> TokenStream {
    format!("{}Owner", name).as_code()
}

pub fn make_handle_owner_name_string(name: &str) -> String {
    format!("{}Owner", name)
}

#[derive(Clone, Copy)]
pub enum FieldContext {
    Member,
    FunctionParam,
}

#[derive(Clone, Copy)]
pub enum WithLifetime<'a> {
    Yes(&'a str),
    No,
}

impl<'a> From<&'a str> for WithLifetime<'a> {
    fn from(s: &'a str) -> Self {
        WithLifetime::Yes(s)
    }
}

pub fn c_type(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> Ty {
    pipe!{ ty = Ty::new() =>
        STAGE ty.basetype(field.basetype.as_str());
        WHEN global_data::uses_lifetime(field.basetype.as_str()) =>
        {
            match with_lifetime {
                WithLifetime::Yes(lifetime) => ty.param(Lifetime::from(lifetime)),
                WithLifetime::No => ty,
            }
        }
        DONE WHEN matches!(field.array, Some(vkxml::ArrayType::Static)) =>
        {
            let size = field
                .size
                .as_ref()
                .or_else(|| field.size_enumref.as_ref())
                .expect("error: field should have size");
            let ty = ty.to_array(ArrayType::array(size));
            match context {
                FieldContext::Member => {
                    // wrap char array in custum type to impl Debug printing
                    if field.basetype == "char" {
                        Ty::new()
                            .basetype("ArrayString")
                            .param(ty)
                    }
                    else {
                        ty
                    }
                }
                FieldContext::FunctionParam =>
                    Ty::new().basetype("Ref")
                    .param(ty),
            }
        }
        DONE WHEN matches!(field.array, Some(vkxml::ArrayType::Dynamic)) =>
        {
            match &field.reference {
                Some(r) => match r {
                    vkxml::ReferenceType::Pointer => {
                        if field.is_const {
                            Ty::new().basetype("Array").param(ty)
                        } else {
                            Ty::new().basetype("ArrayMut").param(ty)
                        }
                    }
                    vkxml::ReferenceType::PointerToPointer => {
                        unimplemented!("unimplemented c_type Array PointerToPointer");
                        //eprintln!("PointerToPointer: {}: {}", field_name_expected(field), field.basetype.as_str());
                    }
                    vkxml::ReferenceType::PointerToConstPointer => {
                        if field.is_const {
                            // TODO a special case fro string arrays would probably be good
                            Ty::new().basetype("Array")
                                .param(ty.pointer(Pointer::Const))
                        } else {
                            unimplemented!("unimplemented c_type Array PointerToConstPointer (Mut)");
                        }
                    }
                },
                None => ty,
            }
        }
        DONE WHEN matches!(field.array, None) =>
        {
            match &field.reference {
                Some(r) => match r {
                    vkxml::ReferenceType::Pointer => {
                        if field.is_const {
                            Ty::new().basetype("Ref").param(ty)
                        } else {
                            Ty::new().basetype("RefMut").param(ty)
                        }
                    }
                    vkxml::ReferenceType::PointerToPointer => {
                        assert!(field.is_const == false);
                        Ty::new().basetype("RefMut")
                            .param(ty.pointer(Pointer::Mut))
                    }
                    vkxml::ReferenceType::PointerToConstPointer => {
                        unimplemented!("unimplemented c_type Ref PointerToConstPointer (Const/Mut)");
                    }
                },
                None => ty,
            }
        }
    }
}

pub fn c_field(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> Field {
    Field::new(case::camel_to_snake(field_name_expected(field)), c_type(field, with_lifetime, context))
}

pub struct Rtype<'a> {
    field: &'a vkxml::Field,
    param_lifetime: WithLifetime<'a>,
    ref_lifetime: WithLifetime<'a>,
    context: FieldContext,
    container: &'a str,
    allow_optional: bool,
}

impl<'a> Rtype<'a> {
    pub fn new(field: &'a vkxml::Field, container: &'a str) -> Self {
        Self {
            field,
            container,
            param_lifetime: WithLifetime::No,
            ref_lifetime: WithLifetime::No,
            context: FieldContext::FunctionParam, // FieldContext Member is the odd one out in c
            allow_optional: true,
        }
    }
    pub fn param_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.param_lifetime = lifetime.into();
        self
    }
    pub fn ref_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.ref_lifetime = lifetime.into();
        self
    }
    pub fn context(mut self, context: FieldContext) -> Self {
        self.context = context;
        self
    }
    pub fn allow_optional(mut self, allow: bool) -> Self {
        self.allow_optional = allow;
        self
    }
    pub fn as_field(&self) -> Field {
        Field::new(case::camel_to_snake(field_name_expected(self.field)), self.as_ty())
    }
    pub fn as_ty(&self) -> Ty {
        let field = self.field;
        let container = self.container;
        let param_lifetime = self.param_lifetime;
        let context = self.context;
        let allow_optional = self.allow_optional;

        let lifetime = || match self.ref_lifetime {
            WithLifetime::Yes(lifetime) => Lifetime::from(lifetime),
            WithLifetime::No => Lifetime::from("'_"),
        };

        pipe!{ ty = Ty::new() =>
            STAGE ty.basetype(field.basetype.as_str());
            WHEN global_data::uses_lifetime(field.basetype.as_str()) =>
            {
                match param_lifetime {
                    WithLifetime::Yes(lifetime) => ty.param(Lifetime::from(lifetime)),
                    WithLifetime::No => ty,
                }
            }
            WHEN global_data::is_externsync(container, field) =>
            {
                Ty::new().basetype("MutHandle")
                    .param(ty)
            }
            WHEN matches!(field.array, Some(vkxml::ArrayType::Static)) =>
            {
                let size = field
                    .size
                    .as_ref()
                    .or_else(|| field.size_enumref.as_ref())
                    .expect("error: field should have size");
                let ty = ty.to_array(ArrayType::array(size));
                match context {
                    FieldContext::Member => ty,

                    // assuming never mut for static size arrays
                    FieldContext::FunctionParam => ty.lifetime(lifetime()).reference(true),
                }
            }
            WHEN matches!(field.array, Some(vkxml::ArrayType::Dynamic)) =>
            {
                match &field.reference {
                    Some(r) => match r {
                        vkxml::ReferenceType::Pointer => {
                            if field.is_const {
                                if field.basetype.as_str() == "char" {
                                    ty.basetype("MyStr")
                                        .param(lifetime())
                                }
                                else {
                                    ty.to_array(ArrayType::Slice)
                                        .lifetime(lifetime())
                                        .reference(true)
                                }
                            } else {
                                ty.to_array(ArrayType::Slice)
                                    .lifetime(lifetime())
                                    .reference(true)
                                    .mutable(true)
                            }
                        }
                        vkxml::ReferenceType::PointerToPointer => unimplemented!("unimplemented rust array PointerToPointer"),
                        vkxml::ReferenceType::PointerToConstPointer => {
                            if field.is_const {
                                let param = if field.basetype.as_str() == "char" {
                                    ty.basetype("MyStr")
                                        .param(lifetime())
                                }
                                else {
                                    ty.pointer(Pointer::Const)
                                };

                                Ty::new()
                                    .basetype("ArrayArray")
                                    .lifetime(lifetime())
                                    .reference(true)
                                    .param(param)
                                    //quote!(&ArrayArray<*const #basetype>)
                                    // TODO find a better type for this
                            } else {
                                unimplemented!("unimplemented rust array mut PointerToConstPointer")
                            }
                        }
                    },
                    None => unreachable!("shouldn't reach this point for makeing rust array type"),
                }
            }
            WHEN matches!(field.array, None) =>
            {
                match &field.reference {
                    Some(r) => match r {
                        vkxml::ReferenceType::Pointer => {
                            if field.is_const {
                                ty.reference(true)
                                    .lifetime(lifetime())
                            } else {
                                ty.reference(true)
                                    .mutable(true)
                                    .lifetime(lifetime())
                            }
                        }
                        vkxml::ReferenceType::PointerToPointer => unimplemented!("unimplemented rust ref PointerToPointer"),
                        vkxml::ReferenceType::PointerToConstPointer => {
                            if field.is_const {
                                unimplemented!("unimplemented rust ref const PointerToConstPointer")
                            } else {
                                unimplemented!("unimplemented rust ref mut PointerToConstPointer")
                            }
                        }
                    },
                    None => ty,
                }
            }
            WHEN is_optional(field)
                && (matches!(context, FieldContext::FunctionParam) || matches!(field.reference, Some(_)))
                && allow_optional
            => {
                Ty::new()
                    .basetype("Option")
                    .param(ty)
            }
            //WHEN field.optional.as_ref().map(|opt|opt.split(',').next() == Some("true")).unwrap_or(false) =>
            //{
            //    match &field.reference {
            //        Some(_) => {
            //            eprintln!("POINTER in : {}", container);
            //            eprintln!("field: {}", field_name_expected(field));
            //        }
            //        None => {
            //            eprintln!("NON POINTER in: {}", container);
            //            eprintln!("field: {}", field_name_expected(field));
            //        }
            //    }
            //    eprintln!();
            //    ty
            //}
        }
    }
}

impl ToTokens for Rtype<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ty().to_tokens(tokens);
    }
}

pub fn r_field(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext, container: &str) -> Field {
    let mut ty = Rtype::new(field, container)
        .context(context)
        .param_lifetime(with_lifetime);
    Field::new(case::camel_to_snake(field_name_expected(field)), ty.as_ty())
}

pub fn r_return_type(field: &vkxml::Field, with_lifetime: WithLifetime) -> Ty {
    let basetype_str = field.basetype.as_str();
    pipe!{ ty = Ty::new() =>
        STAGE {
            if global_data::is_handle(basetype_str) {
                ty.basetype(make_handle_owner_name(basetype_str))
            }
            else {
                ty.basetype(basetype_str)
            }
        }
        WHEN global_data::uses_lifetime(basetype_str) => {
            match with_lifetime {
                WithLifetime::Yes(lifetime) => ty.param(Lifetime::from(lifetime)),
                WithLifetime::No => ty,
            }
        }
        STAGE {
            match field.reference {
                Some(vkxml::ReferenceType::Pointer) => {
                    if field.size.is_some() {
                        Ty::new()
                            .basetype("Vec")
                            .param(ty)
                    }
                    else {
                        ty
                    }
                }
                Some(vkxml::ReferenceType::PointerToPointer) => {
                    ty.pointer(Pointer::Mut)
                }
                Some(vkxml::ReferenceType::PointerToConstPointer) => {
                    panic!("error: PointerToConstPointer in return type")
                }
                None => {
                    ty
                }
            }
        }
    }
}

pub fn is_optional(field: &vkxml::Field) -> bool {
    // optional is a comma seperated list of booleans
    // if the first boolean is true, then is_optional returns true
    if field.optional.as_ref()
        .map(|opt|opt.split(',').next() == Some("true")).unwrap_or(false) {
            true
        }
    // if a type is a pointer and has noautovalidity, then we assume the pointer can be NULL
    // and is_optional is true
    else if field.auto_validity == false && matches!(field.reference, Some(vkxml::ReferenceType::Pointer)) {
        true
    }
    else {
        false
    }
}

pub fn must_init(field: &vkxml::Field) -> bool {
    ! is_optional(field)
}

pub fn ctype_to_rtype(type_name: &str) -> String {
    if type_name == "VkResult" {
        return "VkResultRaw".to_string();
    }
    match type_name {
        "uint8_t" => "u8",
        "uint16_t" => "u16",
        "uint32_t" => "u32",
        "uint64_t" => "u64",
        "int8_t" => "i8",
        "int16_t" => "i16",
        "int32_t" => "i32",
        "int64_t" => "i64",
        "size_t" => "usize",
        "int" => "c_int",
        "void" => "c_void",
        "char" => "c_char",
        "float" => "f32",
        "long" => "c_ulong",
        "type" => "ty",
        x if x.starts_with("Vk") => &type_name[2..],
        x if x.starts_with("vk_cmd_") => &type_name[7..],
        x if x.starts_with("vk_") => &type_name[3..],
        x if x.starts_with("vk") => &type_name[2..],
        x if x.starts_with("VK_") => &type_name[3..],
        _ => type_name,
    }.replace("FlagBits", "Flags")
}

pub fn normalize_flag_names(name: &str) -> String {
    name.replace("FlagBits", "Flags")
}

macro_rules! one_option {

    ( $( $val:expr , $f:expr );+ $(;)* ) => {

        if false {
            unreachable!();
        }
            $( else if let Some(v) = $val {
                $f(v)
            })+
        else {
            panic!("error: reached end of one_option");
        }

    }

}

pub fn find_in_slice<T, F>(slice: &[T], f: F) -> Option<&T> where F: Fn(&T) -> bool {
    for val in slice.iter() {
        if f(val) {
            return Some(val);
        }
    }
    None
}

pub fn is_extension_name(name: &str) -> bool {
    // extension names should end with _EXTENSION_NAME according to the vulkan spec style guide
    // also need to check for ends_with("_NAME") because of an ANDROID extension which failed to follow the proper naming convention
    //      (hopfully no extension defines a const that ends with _NAME other than the ANDROID extension name)
    name.ends_with("_EXTENSION_NAME") || name.ends_with("_NAME")
}

pub fn extension_loader_name(extension_name: &str) -> String {
    format!("{}_loader", extension_name)
}

pub fn platform_specific_types() -> TokenStream {
    quote! {
        pub type RROutput = c_ulong;
        pub type VisualID = c_uint;
        pub type Display = *const c_void;
        pub type Window = c_ulong;
        #[allow(non_camel_case_types)]
        pub type xcb_connection_t = *const c_void;
        #[allow(non_camel_case_types)]
        pub type xcb_window_t = u32;
        #[allow(non_camel_case_types)]
        pub type xcb_visualid_t = *const c_void;
        pub type MirConnection = *const c_void;
        pub type MirSurface = *const c_void;
        pub type HINSTANCE = *const c_void;
        pub type HWND = *const c_void;
        #[allow(non_camel_case_types)]
        pub type wl_display = c_void;
        #[allow(non_camel_case_types)]
        pub type wl_surface = c_void;
        pub type HANDLE = *mut c_void;
        pub type DWORD = c_ulong;
        pub type LPCWSTR = *const u16;
        #[allow(non_camel_case_types)]
        pub type zx_handle_t = u32;

        // FIXME: Platform specific types that should come from a library id:0
        // typedefs are only here so that the code compiles for now
        #[allow(non_camel_case_types)]
        pub type SECURITY_ATTRIBUTES = ();
        // Opage types
        pub type ANativeWindow = c_void;
        pub type AHardwareBuffer = c_void;

        // NOTE These type are included only for compilation purposes
        // These types should NOT be used because they are no necessarily
        // the correct type definitions (i.e. just c_void by default)
        pub type GgpStreamDescriptor = *const c_void;
        pub type CAMetalLayer = *const c_void;
        pub type GgpFrameToken = *const c_void;
        pub type HMONITOR = *const c_void;
    }
}

pub mod case {

    //fn peek_check<I: Iterator<Item=char>, P: std::iter::Peekable<I>>(p: &mut P) -> bool {
    //}

    pub fn camel_to_snake(s: &str) -> String {

        let mut out = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_lowercase() && chars.peek().map_or(false, |c| c.is_uppercase()) {
                out.extend(c.to_lowercase());
                out.push('_');
            }
            else if c.is_alphabetic() && chars.peek().map_or(false, |c| c.is_numeric()) {
                out.extend(c.to_lowercase());
                out.push('_');
            }
            else if c.is_numeric() && chars.peek().map_or(false, |c| c.is_alphabetic())
                && chars.peek().map_or(false, |c| *c != 'D')
                {
                out.push(c);
                out.push('_');
            }
            else {
                out.extend(c.to_lowercase());
            }
        }

        out
    }
}
