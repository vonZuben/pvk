
use quote::quote;
use quote::ToTokens;

use proc_macro2::{TokenStream};

use std::collections::HashMap;
use std::hash::Hash;

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

pub struct VecMap<K, V> {
    vec: Vec<V>,
    map: HashMap<K, usize>,
}

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
            map: Default::default(),
        }
    }
}

impl<K: Eq + Hash, V> VecMap<K, V> {
    pub fn push(&mut self, key: K, val: V) {
        assert!(self.map.insert(key, self.vec.len()).is_none());
        self.vec.push(val);
    }
    pub fn extend(&mut self, items: impl IntoIterator<Item=(K, V)>) {
        for (key, value) in items.into_iter() {
            self.push(key, value);
        }
    }
    pub fn get(&self, key: K) -> Option<&V> {
        let index = self.map.get(&key)?;
        unsafe { Some(self.vec.get_unchecked(*index)) }
    }
    pub fn get_mut_or_default(&mut self, key: K, default: V) -> &mut V {
        match self.map.get(&key) {
            Some(index) => {
                unsafe { self.vec.get_unchecked_mut(*index) }
            }
            None => {
                self.push(key, default);
                self.vec.last_mut().unwrap() // unwrap since we know we just pushed a value
            }
        }
    }
    pub fn contains_or_default(&mut self, key: K, default: V) {
        let _ = self.get_mut_or_default(key, default);
    }
        self.vec.iter()
    }
}

// pub fn structure_type_name<'a>(field: &'a vkxml::Field) -> &'a str {
//     let raw_stype = field.type_enums.as_ref().expect("error: sType with no provided value, or not sType field");
//     &raw_stype[18..] // cut off the "VK_STRUCTURE_TYPE_" from the begining
// }

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

// pub fn is_extension_name(name: &str) -> bool {
//     // extension names should end with _EXTENSION_NAME according to the vulkan spec style guide
//     // also need to check for ends_with("_NAME") because of an ANDROID extension which failed to follow the proper naming convention
//     //      (hopfully no extension defines a const that ends with _NAME other than the ANDROID extension name)
//     name.ends_with("_EXTENSION_NAME") || name.ends_with("_NAME")
// }

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
