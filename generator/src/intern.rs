use std::collections::HashSet;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Once;
use std::borrow::Cow;
use std::sync::atomic::{self, AtomicBool};

use std::cmp::{PartialEq, Eq};

use std::hash::Hash;

static mut INTERNER: Option<Interner> = None;
static mut LOCKED: AtomicBool = AtomicBool::new(false);

pub struct Interner {
    strings: HashSet<String>,
}

impl Interner {
    // should only be called once by one thread
    pub unsafe fn init() {
        INTERNER = Some(Interner{ strings: HashSet::new() });
    }
    // should only be called by one thread at a time, and should ensure init called before
    pub fn intern<'a>(s: impl Into<Cow<'a, str>>) -> Istring {

        // This is using some sketchy cheap method of ensuring only one thread is interning at a time
        // panic if already being used
        // this is only intended to be used in single thread anyway, but better safe than sorry (who knows whats in future)
        unsafe { LOCKED.compare_exchange(false, true, atomic::Ordering::Acquire, atomic::Ordering::Acquire).unwrap(); }

        let s = s.into();
        let mut inner;
        unsafe {
            inner = INTERNER.as_mut().unwrap();
        }
        let ret = match inner.strings.get(s.as_ref()) {
            Some(s) => Istring::new(s.as_str()),
            None => {
                let s = s.into_owned();
                let ptr: *const str = s.as_str();
                assert!(inner.strings.insert(s));
                Istring::new(ptr)
            }
        };

        unsafe { LOCKED.compare_exchange(true, false, atomic::Ordering::Release, atomic::Ordering::Relaxed).unwrap(); }

        ret
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Istring {
    ptr: *const str,
}

impl Istring {
    fn new(ptr: *const str) -> Self {
        Self {
            ptr,
        }
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
        unsafe { Interner::init(); }
        let i1 = Interner::intern("hey");
        let i2 = Interner::intern("hey");
        assert_eq!(i1, i2);
        println!("{:?}", i1);
        println!("{:?}", i2);
    }
}