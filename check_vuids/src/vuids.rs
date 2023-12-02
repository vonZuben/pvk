use std::collections::HashMap;

const VUIDS: &'static str = include_str!(concat!(env!("OUT_DIR"), "/vuids.txt"));

type Target = &'static str;
type Vuid = &'static str;
type Description = &'static str;

#[allow(unused)]
pub struct VuidCollection {
    collection: HashMap<Target, VecMap<Vuid, Description>>,
}

impl VuidCollection {
    // pub fn new() -> Self {
    //     let src = get_vuids();

    //     let collection = src
    //         .iter()
    //         .map(|group| (group.0, VecMap::from_iter(group.1.iter().copied())));

    //     Self {
    //         collection: collection.collect(),
    //     }
    // }
    pub fn get_target(&self, target: Target) -> Option<&dyn TargetVuids> {
        fn coerce(t: &impl TargetVuids) -> &dyn TargetVuids {
            t
        }
        self.collection.get(target).map(coerce)
    }
}

pub trait TargetVuids {
    fn get_description(&self, vuid: Vuid) -> Option<Description>;
    fn iter(&self) -> std::slice::Iter<Description>;
}

struct VecMap<K, V> {
    vec: Vec<V>,
    map: HashMap<K, usize>,
}

impl TargetVuids for VecMap<Vuid, Description> {
    fn get_description(&self, vuid: Vuid) -> Option<Description> {
        self.get(vuid).copied()
    }
    fn iter(&self) -> std::slice::Iter<Description> {
        self.iter()
    }
}

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
            map: Default::default(),
        }
    }
}

impl<K: std::cmp::Eq + std::hash::Hash, V> VecMap<K, V> {
    fn push(&mut self, key: K, val: V) {
        match self.map.insert(key, self.vec.len()) {
            Some(_) => panic!("error: trying to put duplicate item in VecMap"),
            None => {} // good
        }
        self.vec.push(val);
    }
    fn from_iter(items: impl IntoIterator<Item = (K, V)>) -> Self {
        let mut this = Self::default();
        for (key, value) in items.into_iter() {
            this.push(key, value);
        }
        this
    }
    fn get(&self, key: K) -> Option<&V> {
        let index = self.map.get(&key)?;
        unsafe { Some(self.vec.get_unchecked(*index)) }
    }
}

impl<K, V> VecMap<K, V> {
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, V> {
        self.vec.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a VecMap<K, V> {
    type Item = &'a V;

    type IntoIter = std::slice::Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
