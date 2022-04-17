//! # SAFETY
//! It is important that all structs here are repr(transparent)
//! This allows using cheap transmutes and ptr casts between the 
//! inner and wrapper types
//! 
//! The Wrapper type is only there to prevent all access to the inner type.
//! i.e. the wrapper allows all reads via Deref trait, but the wrapper
//! is carful about what writes are allowed if any

use std::ops::Deref;

use vk_safe_sys as vk;

macro_rules! struct_wrapper {
    (
        $name:ident
    ) => {
        #[repr(transparent)]
        pub struct $name {
            inner: vk::$name,
        }
        
        impl Deref for $name {
            type Target = vk::$name;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };
}

struct_wrapper!(ExtensionProperties);

struct_wrapper!(LayerProperties);