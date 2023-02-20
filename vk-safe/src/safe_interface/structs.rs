//! # SAFETY
//! It is important that all structs here are repr(transparent)
//! This allows using cheap transmutes and ptr casts between the
//! inner and wrapper types
//!
//! The Wrapper type is only there to prevent all access to the inner type.
//! i.e. the wrapper allows all reads via Deref trait, but the wrapper
//! is carful about what writes are allowed if any

use std::marker::PhantomData;
use std::ops::Deref;
use std::ffi::CStr;

use crate::utils::VkVersion;

use vk_safe_sys as vk;

// Use this to create wrappers around simple structs
macro_rules! simple_struct_wrapper {
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

fn str_len(s: &[std::os::raw::c_char]) -> usize {
    s.iter().take_while(|&&c| c != 0).count()
}

macro_rules! get_str {
    (
        $name:ident
    ) => {
        pub fn $name(&self) -> &str {
            let unchecked_utf8;
            unsafe {
                unchecked_utf8 = std::slice::from_raw_parts(self.inner.$name.as_ptr().cast(), str_len(&self.inner.$name));
            }
            std::str::from_utf8(unchecked_utf8).expect("vk safe interface internal error: string from Vulkan implementation is not proper utf8")
        }
    };
}

//===========ExtensionProperties
simple_struct_wrapper!(ExtensionProperties);

impl ExtensionProperties {
    get_str!(extension_name);
}

//===========LayerProperties
simple_struct_wrapper!(LayerProperties);

impl LayerProperties {
    get_str!(layer_name);
    get_str!(description);
}
