
use syn::{Expr};

use quote::quote;
use quote::ToTokens;

use proc_macro2::{TokenStream};

pub trait StrAsCode {
    fn as_code(&self) -> TokenStream;
}

// This implementation is intended to convert any string
// into valid tokens
// If you simply want a literal string then don't use this
impl<T> StrAsCode for T where T: AsRef<str> {
    fn as_code(&self) -> TokenStream {
        let rstr = ctype_to_rtype(self.as_ref());
        syn::parse_str::<Expr>(&rstr)
            .expect(format!("error: can't parse {{{}}} as an expresion", &rstr).as_ref())
            .into_token_stream()
    }
}

pub fn get_reference_type(field: &vkxml::Field) -> TokenStream {
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

pub fn handle_field(field: &vkxml::Field) -> TokenStream {

    // if there is no name, then "field" is set as a default name
    // maybe change this later
    let name = field.name.as_ref().map_or(quote!(field), |v| v.as_code());

    let field_type = make_field_type(field);

    quote!{
        #name : #field_type
    }

}

pub fn make_field_type(field: &vkxml::Field) -> TokenStream {
    let basetype = field.basetype.as_code();
    let ref_type = get_reference_type(&field);

    field.array.as_ref().and_then(|a| match a {
        vkxml::ArrayType::Dynamic => None, // if dynamic, then there will be a reference type
        vkxml::ArrayType::Static => {
            let size = field
                .size
                .as_ref()
                .or_else(|| field.size_enumref.as_ref())
                .expect("error: field should have size");
            let size = size.as_code();
            Some( quote!( [ #basetype ; #size ] ) )
        },
    })
    .unwrap_or( quote!( #ref_type #basetype ) )

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
