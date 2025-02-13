use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;

use std::sync::Mutex;

use std::cmp::{Eq, PartialEq};

use std::hash::Hash;

static INTERNER: Mutex<Option<Interner>> = Mutex::new(None);

pub struct Interner {
    strings: HashSet<String>,
}

impl Interner {
    // should only be called once by one thread
    pub unsafe fn init() {
        let mut interner = INTERNER.lock().unwrap();
        if interner.is_none() {
            *interner = Some(Interner {
                strings: HashSet::new(),
            });
        }
    }
    // should ensure init called before calling this
    pub fn intern<'a>(s: impl Into<Cow<'a, str>>) -> Istring {
        let s = s.into();

        let mut interner = INTERNER.lock().unwrap();
        let inner = interner.as_mut().unwrap();

        match inner.strings.get(s.as_ref()) {
            Some(s) => Istring::new(s.as_str()),
            None => {
                let s = s.into_owned();
                let ptr: *const str = s.as_str();
                assert!(inner.strings.insert(s));
                Istring::new(ptr)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Istring {
    ptr: *const str,
}

impl Istring {
    fn new(ptr: *const str) -> Self {
        Self { ptr }
    }
    pub fn get(&self) -> &str {
        unsafe { &*self.ptr }
    }
}

impl fmt::Display for Istring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { &*self.ptr })
    }
}

impl fmt::Debug for Istring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Istring")
            .field("string", unsafe { &&*self.ptr })
            .field("ptr", &self.ptr)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn t1() {
        unsafe {
            Interner::init();
        }
        let i1 = Interner::intern("hey");
        let i2 = Interner::intern("hey");
        assert_eq!(i1, i2);
        println!("{:?}", i1);
        println!("{:?}", i2);
    }
}
