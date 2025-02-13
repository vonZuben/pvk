use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

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
impl<T> StrAsCode for T
where
    T: AsRef<str>,
{
    fn as_code(&self) -> TokenWrapper {
        let rstr = ctype_to_rtype(self.as_ref());
        // rstr.parse()
        //     .expect(format!("error: can't parse {{{}}} as TokenStream", &rstr).as_ref())
        TokenWrapper(rstr.into())
    }
}

/// Hashmap with Vec ordering
///
/// use push instead of insert
pub struct VecMap<K, V> {
    vec: Vec<V>,
    map: HashMap<K, usize>,
    key_copies: Option<Vec<K>>,
}

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
            map: Default::default(),
            key_copies: Default::default(),
        }
    }
}

impl<K: Eq + Hash, V> VecMap<K, V> {
    /// push new element with key
    /// panics if the same key is used more than once
    pub fn push(&mut self, key: K, val: V) {
        match self.map.insert(key, self.vec.len()) {
            Some(_) => panic!("error: trying to put duplicate item in VecMap"),
            None => {} // good
        }
        self.vec.push(val);
    }
    /// extend the VecMap from an iterator of (key, value) tuples
    pub fn extend(&mut self, items: impl IntoIterator<Item = (K, V)>) {
        for (key, value) in items.into_iter() {
            self.push(key, value);
        }
    }
    /// get reference to element with key
    pub fn get(&self, key: K) -> Option<&V> {
        let index = self.map.get(&key)?;
        unsafe { Some(self.vec.get_unchecked(*index)) }
    }
    /// get mutable reference to element with key
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let index = self.map.get(&key)?;
        unsafe { Some(self.vec.get_unchecked_mut(*index)) }
    }
    /// get mutable reference to existing element or set default element with key
    pub fn get_mut_or_default(&mut self, key: K, default: V) -> &mut V {
        match self.map.get(&key) {
            Some(index) => unsafe { self.vec.get_unchecked_mut(*index) },
            None => {
                self.push(key, default);
                self.vec.last_mut().unwrap() // unwrap since we know we just pushed a value
            }
        }
    }
    /// get mutable reference to existing element or default element using closure with key
    pub fn get_mut_or_default_with(&mut self, key: K, default: impl FnOnce() -> V) -> &mut V {
        match self.map.get(&key) {
            Some(index) => unsafe { self.vec.get_unchecked_mut(*index) },
            None => {
                self.push(key, default());
                self.vec.last_mut().unwrap() // unwrap since we know we just pushed a value
            }
        }
    }
    /// ensures the VecMap contains a value with key, and sets it if not
    pub fn contains_or_default(&mut self, key: K, default: V) {
        let _ = self.get_mut_or_default(key, default);
    }
    /// like the HashMap entry api
    pub fn entry<'a>(&'a mut self, key: K) -> VecMapEntry<'a, K, V> {
        match self.map.get(&key) {
            Some(index) => VecMapEntry::Occupied(self, *index),
            None => VecMapEntry::Empty(self, key),
        }
    }
}

impl<K: Eq + Hash + Copy, V> VecMap<K, V> {
    /// If a key is copyable, use this method to push the key and value in a way where the keys will also be iterable in insertion order
    pub fn push_copy_key(&mut self, key: K, val: V) {
        self.push(key, val);
        match self.key_copies {
            Some(ref mut keys) => keys.push(key),
            None => self.key_copies = Some(vec![key]),
        }
    }

    /// iterate of the keys and values in insertion order
    pub fn ordered_key_value_iter<'a>(&'a self) -> Option<impl Iterator<Item = (K, &V)> + 'a> {
        match self.key_copies {
            None => None,
            Some(ref keys) => Some(keys.iter().copied().zip(self.vec.iter())),
        }
    }
}

impl<K, V> VecMap<K, V> {
    /// iterate over the elements of the VecMap in insertion order
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, V> {
        self.vec.iter()
    }
    /// get reference to the last element pushed to the VecMap
    pub fn last(&self) -> Option<&V> {
        self.vec.last()
    }
    /// get mutable reference to the last element pushed to the VecMap
    pub fn last_mut(&mut self) -> Option<&mut V> {
        self.vec.last_mut()
    }
}

impl<'a, K, V> IntoIterator for &'a VecMap<K, V> {
    type Item = &'a V;

    type IntoIter = std::slice::Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub enum VecMapEntry<'a, K, V> {
    Occupied(&'a mut VecMap<K, V>, usize),
    Empty(&'a mut VecMap<K, V>, K),
}

impl<'a, K: Eq + Hash, V> VecMapEntry<'a, K, V> {
    #[allow(unused)]
    pub fn or_insert_with(self, f: impl FnOnce() -> V) -> &'a mut V {
        match self {
            VecMapEntry::Occupied(vm, index) => unsafe { vm.vec.get_unchecked_mut(index) },
            VecMapEntry::Empty(vm, key) => {
                vm.push(key, f());
                vm.last_mut().unwrap()
            }
        }
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
        } else {
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
    pub fn normalize(&self) -> &str {
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

pub mod case {

    pub fn camel_to_snake(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_lowercase() && chars.peek().map_or(false, |c| c.is_uppercase()) {
                out.extend(c.to_lowercase());
                out.push('_');
            } else if c.is_alphabetic() && chars.peek().map_or(false, |c| c.is_numeric()) {
                out.extend(c.to_lowercase());
                out.push('_');
            } else if c.is_numeric()
                && chars.peek().map_or(false, |c| c.is_alphabetic())
                && chars.peek().map_or(false, |c| *c != 'D')
            {
                out.push(c);
                out.push('_');
            } else {
                out.extend(c.to_lowercase());
            }
        }

        out
    }

    pub fn normalize(s: &str) -> String {
        s.chars()
            .filter_map(|c| {
                assert!(c.is_ascii());
                if c == '_' {
                    None
                } else {
                    Some(c.to_ascii_lowercase())
                }
            })
            .collect()
    }
}
