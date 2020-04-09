
use syn::{Expr};

use quote::quote;
use quote::ToTokens;

use proc_macro2::{TokenStream};

use crate::ty;
use crate::global_data;

//pub fn create_entry_code() -> TokenStream {
//
//    quote!{
//        //pub fn create_instance(
//    }
//
//}

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

fn make_basetype(field: &vkxml::Field, with_lifetime: WithLifetime) -> TokenStream {
    let basetype = field.basetype.as_code();

    use WithLifetime::*;
    let lifetime = match with_lifetime {
        Yes => Some( global_data::lifetime(field.basetype.as_str()) ),
        No => None,
    };

    quote!( #basetype #lifetime )
}

pub fn make_c_reference_type(field: &vkxml::Field) -> TokenStream {
     match &field.reference {
        Some(r) => match r {
            vkxml::ReferenceType::Pointer => {
                if field.is_const {
                    quote!(*const)
                } else {
                    quote!(*mut)
                }
            }
            vkxml::ReferenceType::PointerToPointer => quote!(*mut *mut),
            vkxml::ReferenceType::PointerToConstPointer => {
                if field.is_const {
                    quote!(*const *const)
                } else {
                    quote!(*mut *const)
                }
            }
        },
        None => quote!(),
    }
}

pub enum FieldContext {
    Member,
    FunctionParam,
}

#[derive(Clone, Copy)]
pub enum WithLifetime {
    Yes,
    No,
}

pub fn make_c_field(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> TokenStream {

    // if there is no name, then "field" is set as a default name
    // maybe change this later
    let name = field.name.as_ref().map_or(quote!(un_named_field), |v| v.as_code());

    let field_type = make_c_type(field, with_lifetime, context);

    quote!{
        #name : #field_type
    }

}

pub fn make_c_type(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> TokenStream {
    let basetype = make_basetype(field, with_lifetime);
    let ref_type = make_c_reference_type(&field);

    field.array.as_ref().and_then(|a| match a {
        vkxml::ArrayType::Dynamic => {
            Some( quote!(Array<#ref_type #basetype>) )
        }
        vkxml::ArrayType::Static => {
            let size = field
                .size
                .as_ref()
                .or_else(|| field.size_enumref.as_ref())
                .expect("error: field should have size");
            let size = size.as_code();
            match context {
                FieldContext::Member => Some( quote!([#basetype;#size]) ),
                // NOTE: I am assuming that there are never any mut static size arrays
                FieldContext::FunctionParam => Some( quote!(Ref<*const [#basetype;#size]>) ),
            }
        },
    })
    .unwrap_or_else(|| {
        if field.reference.is_some() {
            quote!( Ref<#ref_type #basetype> )
        }
        else {
            quote!( #ref_type #basetype )
        }
    })
}

pub fn c_type(field: &vkxml::Field) -> ty::Ty {

    // raw c types should never need lifetimes
    // or & reference

    let mut ty = ty::Ty::new();

    if field.reference.is_some() {
    }

    unimplemented!()
}

// make rust reference type, but will simply pass the basetype without reference if there is none
pub fn make_rust_reference_type(field: &vkxml::Field, basetype: &TokenStream) -> TokenStream {
    match &field.reference {
        Some(r) => match r {
            vkxml::ReferenceType::Pointer => {
                if field.is_const {
                    quote!(&#basetype)
                } else {
                    quote!(&mut#basetype)
                }
            }
            vkxml::ReferenceType::PointerToPointer => unimplemented!("unimplemented rust ref PointerToPointer"),
            vkxml::ReferenceType::PointerToConstPointer => {
                if field.is_const {
                    //quote!(*const *const)
                    unimplemented!("unimplemented rust ref const PointerToConstPointer")
                } else {
                    //quote!(*mut *const)
                    unimplemented!("unimplemented rust ref mut PointerToConstPointer")
                }
            }
        },
        None => quote!(#basetype),
    }
}

// make rust array type, should only be called for type that have equivelent c pointer types
pub fn make_rust_array_type(field: &vkxml::Field, basetype: &TokenStream) -> TokenStream {
    match &field.reference {
        Some(r) => match r {
            vkxml::ReferenceType::Pointer => {
                if field.is_const {
                    if field.basetype.as_str() == "char" {
                        quote!(&CStr)
                    }
                    else {
                        quote!(&[#basetype])
                    }
                } else {
                    quote!(&mut[#basetype])
                }
            }
            vkxml::ReferenceType::PointerToPointer => unimplemented!("unimplemented rust array PointerToPointer"),
            vkxml::ReferenceType::PointerToConstPointer => {
                if field.is_const {
                    quote!(&ArrayArray<*const #basetype>)
                } else {
                    //quote!(*mut *const)
                    unimplemented!("unimplemented rust array mut PointerToConstPointer")
                }
            }
        },
        None => unreachable!("shouldn't reach this point for makeing rust array type"),
    }
}

pub fn make_rust_field(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> TokenStream {
    // if there is no name, then "field" is set as a default name
    // maybe change this later
    let name = field.name.as_ref().map_or(quote!(un_named_field), |v| v.as_code());

    let field_type = make_rust_type(field, with_lifetime, context);

    quote!{
        #name : #field_type
    }

}

pub fn make_rust_type(field: &vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> TokenStream {
    let basetype = make_basetype(field, with_lifetime);
    field.array.as_ref().and_then(|a| match a {
        vkxml::ArrayType::Dynamic => {
            let ty = make_rust_array_type(field, &basetype);
            Some( quote!(#ty) )
        }
        vkxml::ArrayType::Static => {
            let size = field
                .size
                .as_ref()
                .or_else(|| field.size_enumref.as_ref())
                .expect("error: field should have size");
            let size = size.as_code();
            match context {
                FieldContext::Member => Some( quote!([#basetype;#size]) ),
                // NOTE: I am assuming that there are never any mut static size arrays
                FieldContext::FunctionParam => Some( quote!(&[#basetype;#size]) ),
            }
        },
    })
    .unwrap_or_else(|| {
        let ty = make_rust_reference_type(field, &basetype);
        quote!( #ty )
    })
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
