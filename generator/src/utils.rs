
use quote::quote;
use quote::ToTokens;

use proc_macro2::{TokenStream};

use std::collections::HashMap;
use std::hash::Hash;

use crate::intern::{Interner, Istring};

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
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let index = self.map.get(&key)?;
        unsafe { Some(self.vec.get_unchecked_mut(*index)) }
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
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&'a V> + Clone {
        self.vec.iter()
    }
}

// This is for ensureing all names are handled consistently
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct VkName {
    name: Istring,
}

impl VkName {
    pub fn new(name: &str) -> Self {
        let name = match name {
            x if x.starts_with("Vk") => &x[2..],
            // x if x.starts_with("vk_cmd_") => &type_name[7..],
            x if x.starts_with("vk_") => &x[3..],
            x if x.starts_with("vk") => &x[2..],
            x if x.starts_with("VK_") => &x[3..],
            x => x,
        };
        
        if name.contains("FlagBits") {
            let name = name.replace("FlagBits", "Flags");
            Self {
                name: Interner::intern(name),
            }
        }
        else {
            Self {
                name: Interner::intern(name),
            }
        }
    }
    pub fn as_code(&self) -> TokenStream {
        let this = self;
        quote!( #this )
    }
}

impl ToTokens for VkName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.get().as_code().to_tokens(tokens)
    }
}

impl std::ops::Deref for VkName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.name.get()
    }
}

impl From<&str> for VkName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<&String> for VkName {
    fn from(name: &String) -> Self {
        Self::new(name)
    }
}

// pub fn structure_type_name<'a>(field: &'a vkxml::Field) -> &'a str {
//     let raw_stype = field.type_enums.as_ref().expect("error: sType with no provided value, or not sType field");
//     &raw_stype[18..] // cut off the "VK_STRUCTURE_TYPE_" from the begining
// }

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
        "double" => "f64",
        "long" => "c_ulong",
        "type" => "ty",
        x if x.starts_with("Vk") => &type_name[2..],
        // x if x.starts_with("vk_cmd_") => &type_name[7..],
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
