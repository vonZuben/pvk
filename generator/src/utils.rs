use std::collections::HashMap;
use std::hash::Hash;
use std::fmt;

use crate::intern::{Interner, Istring};

pub struct TokenWrapper(krs_quote::Token);

impl From<krs_quote::Token> for TokenWrapper {
    fn from(t: krs_quote::Token) -> Self {
        Self(t)
    }
}

impl krs_quote::ToTokens for TokenWrapper {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

pub trait StrAsCode {
    fn as_code(&self) -> TokenWrapper;
}

// This implementation is intended to convert any string
// into valid tokens
// If you simply want a literal string then don't use this
impl<T> StrAsCode for T where T: AsRef<str> {
    fn as_code(&self) -> TokenWrapper {
        let rstr = ctype_to_rtype(self.as_ref());
        // rstr.parse()
        //     .expect(format!("error: can't parse {{{}}} as TokenStream", &rstr).as_ref())
        TokenWrapper(rstr.into())
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
        match self.map.insert(key, self.vec.len()) {
            Some(_) => panic!("error: trying to put duplicate item in vecmap"),
            None => {} // good
        }
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

// This is for ensuring all names are handled consistently
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct VkTyName {
    name: Istring,
}

impl VkTyName {
    pub fn new<'a, C: Into<std::borrow::Cow<'a, str>>>(name: C) -> Self {
        let name = name.into();
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
    pub fn as_str(&self) -> &str {
        self
    }
    pub fn as_code(&self) -> TokenWrapper {
        krs_quote::Token::from(self.normalize()).into()
        // let this = self;
        // quote!( #this )
    }
    fn normalize(&self) -> &str {
        ctype_to_rtype(self.name.get())
    }
}

impl krs_quote::ToTokens for VkTyName {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.normalize();
        tokens.push(name);
    }
}

impl fmt::Display for VkTyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.normalize())
    }
}

impl std::ops::Deref for VkTyName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.name.get()
    }
}

impl<'a, C: Into<std::borrow::Cow<'a, str>>> From<C> for VkTyName {
    fn from(name: C) -> Self {
        Self::new(name)
    }
}

pub fn ctype_to_rtype(type_name: &str) -> &str {
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
        x if x.starts_with("vk_") => &type_name[3..],
        x if x.starts_with("vk") => &type_name[2..],
        x if x.starts_with("VK_") => &type_name[3..],
        x => x,
    }
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

pub mod case {

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
