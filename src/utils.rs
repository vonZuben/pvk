
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

pub fn field_name_expected(field: &vkxml::Field) -> &str {
    field.name.as_ref().expect("error: field does not have name when expected").as_str()
}

pub fn make_handle_owner_name(name: &str) -> TokenStream {
    format!("{}Owner", name).as_code()
}

pub fn make_handle_owner_name_string(name: &str) -> String {
    format!("{}Owner", name)
}

pub enum FieldContext {
    Member,
    FunctionParam,
}

#[derive(Clone, Copy)]
pub enum WithLifetime<'a> {
    Yes(&'a str),
    No,
}

macro_rules! is_variant {
    ( $variant:pat, $other:expr ) => {
        {
            match $other {
                $variant => true,
                _ => false,
            }
        }
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
        DONE WHEN is_variant!(Some(vkxml::ArrayType::Static), field.array) =>
        {
            let size = field
                .size
                .as_ref()
                .or_else(|| field.size_enumref.as_ref())
                .expect("error: field should have size");
            let ty = ty.to_array(ArrayType::array(size));
            match context {
                FieldContext::Member => ty,
                FieldContext::FunctionParam =>
                    Ty::new().basetype("Ref")
                    .param(ty.pointer(Pointer::Const)),
            }
        }
        STAGE {
            match &field.reference {
                Some(r) => match r {
                    vkxml::ReferenceType::Pointer => {
                        if field.is_const {
                            ty.pointer(ty::Pointer::Const)
                        } else {
                            ty.pointer(ty::Pointer::Mut)
                        }
                    }
                    vkxml::ReferenceType::PointerToPointer => ty.pointer(ty::Pointer::Mut)
                                                                .pointer(ty::Pointer::Mut),
                    vkxml::ReferenceType::PointerToConstPointer => {
                        if field.is_const {
                            ty.pointer(ty::Pointer::Const)
                              .pointer(ty::Pointer::Const)
                        } else {
                            ty.pointer(ty::Pointer::Mut)
                              .pointer(ty::Pointer::Const)
                        }
                    }
                },
                None => ty,
            }
        }
        DONE WHEN is_variant!(Some(vkxml::ArrayType::Dynamic), field.array) =>
        {
            Ty::new().basetype("Array").param(ty)
        }
        DONE WHEN field.reference.is_some() =>
        {
            Ty::new().basetype("Ref").param(ty)
        }
    }
}

pub fn c_field(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> Field {
    Field::new(field_name_expected(field), c_type(field, with_lifetime, context))
}

pub fn r_type(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext, container: &str) -> Ty {
    pipe!{ ty = Ty::new() =>
        STAGE ty.basetype(field.basetype.as_str());
        WHEN global_data::uses_lifetime(field.basetype.as_str()) =>
        {
            match with_lifetime {
                WithLifetime::Yes(lifetime) => ty.param(Lifetime::from(lifetime)),
                WithLifetime::No => ty,
            }
        }
        WHEN global_data::is_externsync(container, field) =>
        {
            Ty::new().basetype("MutBorrow")
                .param(ty)
        }
        DONE WHEN is_variant!(Some(vkxml::ArrayType::Static), field.array) =>
        {
            let size = field
                .size
                .as_ref()
                .or_else(|| field.size_enumref.as_ref())
                .expect("error: field should have size");
            let ty = ty.to_array(ArrayType::array(size));
            match context {
                FieldContext::Member => ty,
                FieldContext::FunctionParam => ty.reference(true), // assuming never mut for static size arrays
            }
        }
        DONE WHEN is_variant!(Some(vkxml::ArrayType::Dynamic), field.array) =>
        {
            match &field.reference {
                Some(r) => match r {
                    vkxml::ReferenceType::Pointer => {
                        if field.is_const {
                            if field.basetype.as_str() == "char" {
                                ty.basetype("CStr")
                                    .reference(true)
                            }
                            else {
                                ty.to_array(ArrayType::Slice)
                                    .reference(true)
                            }
                        } else {
                            ty.to_array(ArrayType::Slice)
                                .reference(true)
                                .mutable(true)
                        }
                    }
                    vkxml::ReferenceType::PointerToPointer => unimplemented!("unimplemented rust array PointerToPointer"),
                    vkxml::ReferenceType::PointerToConstPointer => {
                        if field.is_const {
                            Ty::new()
                                .basetype("ArrayArray")
                                .reference(true)
                                .param(ty.pointer(Pointer::Const))
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
        STAGE
        {
            match &field.reference {
                Some(r) => match r {
                    vkxml::ReferenceType::Pointer => {
                        if field.is_const {
                            ty.reference(true)
                        } else {
                            ty.reference(true)
                                .mutable(true)
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

    }
}

pub fn r_field(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext, container: &str) -> Field {
    Field::new(field_name_expected(field), r_type(field, with_lifetime, context, container))
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

pub fn ctype_to_rtype(type_name: &str) -> String {
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
        x if x.starts_with("vk_") => &type_name[3..],
        x if x.starts_with("vk") => &type_name[2..],
        x if x.starts_with("VK_") => &type_name[3..],
        _ => type_name,
    }.replace("FlagBits", "Flags")
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
