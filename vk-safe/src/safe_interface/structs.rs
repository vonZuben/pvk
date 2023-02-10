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

//===========InstanceCreateInfo
pub struct InstanceCreateInfo<'a, V: vk_safe_sys::VulkanVersion, E> {
    inner: vk::InstanceCreateInfo,
    _version: PhantomData<V>,
    _extensions: PhantomData<E>,
    _refs: PhantomData<&'a ()>,
}

impl<'a, V: vk_safe_sys::VulkanVersion, E> InstanceCreateInfo<'a, V, E> {
    pub fn new(app_info: &'a ApplicationInfo<'a, V>) -> Self {
        Self {
            inner: vk::InstanceCreateInfo {
                s_type: vk::StructureType::INSTANCE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::InstanceCreateFlags::empty(),
                p_application_info: &app_info.inner,
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: 0,
                pp_enabled_extension_names: std::ptr::null(),
            },
            _version: PhantomData,
            _extensions: PhantomData,
            _refs: PhantomData,
        }
    }

    pub fn extensions<E2>(self, extensions: E2) -> InstanceCreateInfo<'a, V, E2> {
        // let new = InstanceCreateInfo {
        //     _extensions: PhantomData,
        //     inner: self.inner,
        //     _version: self._version,
        //     _refs: self._refs,
        // };
        todo!() // need to set the extension properly, probably need to define extension trait properly
    }
}

//===========ApplicationInfo
pub struct ApplicationInfo<'a, V: vk_safe_sys::VulkanVersion> {
    inner: vk::ApplicationInfo,
    _version: PhantomData<V>,
    _refs: PhantomData<&'a ()>,
}

impl<'a, V: vk_safe_sys::VulkanVersion> ApplicationInfo<'a, V> {
    pub fn new(_version: V) -> Self {
        let version_tuple = V::VersionTriple;
        let version = VkVersion::new(0, version_tuple.0, version_tuple.1, version_tuple.2);
        Self {
            inner: vk::ApplicationInfo {
                s_type: vk::StructureType::APPLICATION_INFO,
                p_next: std::ptr::null(),
                p_application_name: std::ptr::null(),
                application_version: 0,
                p_engine_name: std::ptr::null(),
                engine_version: 0,
                api_version: version.raw(),
            },
            _version: PhantomData,
            _refs: PhantomData,
        }
    }

    pub fn app_name_and_version(mut self, name: &'a CStr, version: VkVersion) -> Self {
        self.inner.p_application_name = name.as_ptr();
        self.inner.application_version = version.raw();
        self
    }

    pub fn engine_name_and_version(mut self, name: &'a CStr, version: VkVersion) -> Self {
        self.inner.p_engine_name = name.as_ptr();
        self.inner.engine_version = version.raw();
        self
    }
}
