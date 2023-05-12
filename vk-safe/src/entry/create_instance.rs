use super::command_impl_prelude::*;

use crate::instance as safe_instance;
use crate::instance::InstanceConfig;
use crate::pretty_version::VkVersion;

use std::mem::MaybeUninit;
use std::marker::PhantomData;
use std::ffi::CStr;

#[derive(Debug)]
pub struct TempError;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateInstance.html
*/
impl_safe_entry_interface! {
CreateInstance {
    pub fn create_instance<C: InstanceConfig>(
        &self,
        create_info: &InstanceCreateInfo<C>,
    ) -> std::result::Result<safe_instance::Instance<C>, TempError> {
        validate_create_instance::Validation::validate();
        let mut instance = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get().get_fptr()(&create_info.inner, None.to_c(), instance.as_mut_ptr());
            if res.is_err() {
                return Err(TempError);
            }
            Ok(safe_instance::Instance::load_commands(instance.assume_init()).map_err(|_|TempError)?)
        }
    }
}}

mod validate_create_instance {
    use vk_safe_sys::validation::CreateInstance::*;

    pub struct Validation;

    impl Validation {
        pub fn validate() {
            validate(Self)
        }
    }

    #[allow(non_upper_case_globals)]
    impl Vuids for Validation {
        const VUID_vkCreateInstance_ppEnabledExtensionNames_01388: () = {
            // for checking at InstanceCreateInfo construction
        };

        const VUID_vkCreateInstance_pCreateInfo_parameter: () = {
            // taken by rust reference, so the pointer is valid, and the structure itself is validated on it's own
        };

        const VUID_vkCreateInstance_pAllocator_parameter: () = {
            // taken by rust reference, so the pointer is valid, and the structure itself is validated on it's own
        };

        const VUID_vkCreateInstance_pInstance_parameter: () = {
            // using MaybeUninit::as_mut_ptr()
        };
    }

    check_vuid_defs!(
        pub const VUID_vkCreateInstance_ppEnabledExtensionNames_01388 : & 'static [ u8 ] = "All required extensions for each extension in the VkInstanceCreateInfo::ppEnabledExtensionNames list must also be present in that list." . as_bytes ( ) ;
        pub const VUID_vkCreateInstance_pCreateInfo_parameter: &'static [u8] =
            "pCreateInfo must be a valid pointer to a valid VkInstanceCreateInfo structure"
                .as_bytes();
        pub const VUID_vkCreateInstance_pAllocator_parameter : & 'static [ u8 ] = "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure" . as_bytes ( ) ;
        pub const VUID_vkCreateInstance_pInstance_parameter: &'static [u8] =
            "pInstance must be a valid pointer to a VkInstance handle".as_bytes();
    );
}

//===========InstanceCreateInfo
pub struct InstanceCreateInfo<'a, C: InstanceConfig> {
    pub(crate) inner: vk::InstanceCreateInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
}

impl<'a, C: InstanceConfig> InstanceCreateInfo<'a, C> {
    pub fn new(app_info: &'a ApplicationInfo<'a, C>) -> Self {
        Self {
            inner: vk::InstanceCreateInfo {
                s_type: vk::VkEnum::from_variant_type(vk::structure_type::INSTANCE_CREATE_INFO),
                p_next: std::ptr::null(),
                flags: unsafe { vk::InstanceCreateFlags::empty() },
                p_application_info: &app_info.inner,
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: 0,
                pp_enabled_extension_names: std::ptr::null(),
            },
            _config: PhantomData,
            _refs: PhantomData,
        }
    }

    // pub fn extensions<E2>(self, extensions: E2) -> InstanceCreateInfo<'a, V, E2> {
    //     // let new = InstanceCreateInfo {
    //     //     _extensions: PhantomData,
    //     //     inner: self.inner,
    //     //     _version: self._version,
    //     //     _refs: self._refs,
    //     // };
    //     todo!() // need to set the extension properly, probably need to define extension trait properly
    // }
}

//===========ApplicationInfo
pub struct ApplicationInfo<'a, C: InstanceConfig> {
    inner: vk::ApplicationInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
}

impl<'a, C: InstanceConfig> ApplicationInfo<'a, C> {
    pub fn new(_config: C) -> Self {
        let version = C::VERSION;
        Self {
            inner: vk::ApplicationInfo {
                s_type: vk::VkEnum::from_variant_type(vk::structure_type::APPLICATION_INFO),
                p_next: std::ptr::null(),
                p_application_name: std::ptr::null(),
                application_version: 0,
                p_engine_name: std::ptr::null(),
                engine_version: 0,
                api_version: version.raw(),
            },
            _config: PhantomData,
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