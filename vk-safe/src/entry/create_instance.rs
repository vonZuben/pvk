use crate::error::Error;
use crate::instance::{Config, InstanceType};
use crate::pretty_version::VkVersion;
use crate::type_conversions::ToC;
use crate::vk_str::VkStr;

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use vk::commands::{LoadCommands, Version};
use vk::has_command::DestroyInstance;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateInstance.html
*/
pub fn create_instance<P, C>(
    create_info: &InstanceCreateInfo<C>,
) -> Result<InstanceType<Config<P, C>>, Error>
where
    C: DestroyInstance<P> + Version + LoadCommands,
{
    check_vuids::check_vuids!(CreateInstance);
    // check_vuid_defs2!(CreateInstance
    //     pub const VUID_vkCreateInstance_ppEnabledExtensionNames_01388 : & 'static [ u8 ] = "All required extensions for each extension in the VkInstanceCreateInfo::ppEnabledExtensionNames list must also be present in that list" . as_bytes ( ) ;
    //     CHECK {
    //          // checked in InstanceCreateInfo construction
    //     }
    //     pub const VUID_vkCreateInstance_pCreateInfo_parameter: &'static [u8] =
    //         "pCreateInfo must be a valid pointer to a valid VkInstanceCreateInfo structure"
    //             .as_bytes();
    //     CHECK {
    //         // taken by rust reference, so the pointer is valid, and the structure itself is validated on it's own
    //     }
    //     pub const VUID_vkCreateInstance_pAllocator_parameter : & 'static [ u8 ] = "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure" . as_bytes ( ) ;
    //     CHECK {
    //         // taken by rust reference, so the pointer is valid, and the structure itself is validated on it's own
    //     }
    //     pub const VUID_vkCreateInstance_pInstance_parameter: &'static [u8] =
    //         "pInstance must be a valid pointer to a VkInstance handle".as_bytes();
    //     CHECK {
    //         // using MaybeUninit::as_mut_ptr()
    //     }
    // );

    // TODO: return proper error for failing to load the command
    let command = super::entry_fn_loader::<vk::CreateInstance>()
        .unwrap()
        .get_fptr();

    let mut instance = MaybeUninit::uninit();
    unsafe {
        let res = command(&create_info.inner, None.to_c(), instance.as_mut_ptr());
        check_raw_err!(res);
        Ok(InstanceType::load_commands(instance.assume_init())?)
    }
}

//===========InstanceCreateInfo
pub struct InstanceCreateInfo<'a, C> {
    pub(crate) inner: vk::InstanceCreateInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
}

impl<'a, C> InstanceCreateInfo<'a, C> {
    pub const fn new(app_info: &'a ApplicationInfo<'a, C>) -> Self {
        check_vuids::check_vuids!(InstanceCreateInfo);

        // check_vuid_defs2!( InstanceCreateInfo
        //     pub const VUID_VkInstanceCreateInfo_sType_sType: &'static [u8] =
        //         "sType must be VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO".as_bytes();
        //     CHECK {
        //         // set below
        //     }
        //     pub const VUID_VkInstanceCreateInfo_pNext_pNext : & 'static [ u8 ] = "Each pNext member of any structure (including this one) in the pNext chain must be either NULL or a pointer to a valid instance of VkDebugReportCallbackCreateInfoEXT, VkDebugUtilsMessengerCreateInfoEXT, VkValidationFeaturesEXT, or VkValidationFlagsEXT" . as_bytes ( ) ;
        //     CHECK {
        //         /*
        //         ===========================================
        //         ===============TODO========================
        //         ===========================================
        //         currently safe since pnext list is disallowed, but when added, this needs to be checked
        //         */
        //     }
        //     pub const VUID_VkInstanceCreateInfo_sType_unique: &'static [u8] =
        //         "The sType value of each struct in the pNext chain must be unique".as_bytes();
        //     CHECK {
        //         /*
        //         ===========================================
        //         ===============TODO========================
        //         ===========================================
        //         currently safe since pnext list is disallowed, but when added, this needs to be checked
        //         */
        //     }
        //     pub const VUID_VkInstanceCreateInfo_flags_zerobitmask: &'static [u8] =
        //         "flags must be 0".as_bytes();
        //     CHECK {
        //         // set below
        //     }
        //     pub const VUID_VkInstanceCreateInfo_pApplicationInfo_parameter : & 'static [ u8 ] = "If pApplicationInfo is not NULL, pApplicationInfo must be a valid pointer to a valid VkApplicationInfo structure" . as_bytes ( ) ;
        //     CHECK {
        //         // app_info is provided by valid reference, and the structure itself is validated at construction
        //     }
        //     pub const VUID_VkInstanceCreateInfo_ppEnabledLayerNames_parameter : & 'static [ u8 ] = "If enabledLayerCount is not 0, ppEnabledLayerNames must be a valid pointer to an array of enabledLayerCount null-terminated UTF-8 strings" . as_bytes ( ) ;
        //     CHECK {
        //         /*
        //         ===========================================
        //         ===============TODO========================
        //         ===========================================
        //         currently layers are not allowed, but when added this should be checked
        //         */
        //     }
        //     pub const VUID_VkInstanceCreateInfo_ppEnabledExtensionNames_parameter : & 'static [ u8 ] = "If enabledExtensionCount is not 0, ppEnabledExtensionNames must be a valid pointer to an array of enabledExtensionCount null-terminated UTF-8 strings" . as_bytes ( ) ;
        //     CHECK {
        //         /*
        //         ===========================================
        //         ===============TODO========================
        //         ===========================================
        //         currently extensions are not allowed, but when added this should be checked
        //         */
        //     }
        // );

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
            _config: PhantomData,
            _refs: PhantomData,
        }
    }
}

//===========ApplicationInfo
pub struct ApplicationInfo<'a, C> {
    inner: vk::ApplicationInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
}

impl<'a> ApplicationInfo<'a, ()> {
    pub const fn new<Commands: Version>() -> ApplicationInfo<'a, Commands> {
        check_vuids::check_vuids!(ApplicationInfo);
        // check_vuid_defs2!( ApplicationInfo
        //     pub const VUID_VkApplicationInfo_sType_sType: &'static [u8] =
        //         "sType must be VK_STRUCTURE_TYPE_APPLICATION_INFO".as_bytes();
        //     CHECK {
        //         // set below
        //     }
        //     pub const VUID_VkApplicationInfo_pNext_pNext: &'static [u8] =
        //         "pNext must be NULL".as_bytes();
        //     CHECK {
        //         // set below
        //     }
        //     pub const VUID_VkApplicationInfo_pApplicationName_parameter : & 'static [ u8 ] = "If pApplicationName is not NULL, pApplicationName must be a null-terminated UTF-8 string" . as_bytes ( ) ;
        //     CHECK {
        //         // VkStr ensures null-terminated UTF-8 string
        //     }
        //     pub const VUID_VkApplicationInfo_pEngineName_parameter: &'static [u8] =
        //         "If pEngineName is not NULL, pEngineName must be a null-terminated UTF-8 string"
        //             .as_bytes();
        //     CHECK {
        //         // VkStr ensures null-terminated UTF-8 string
        //     }
        //     pub const VUID_VkApplicationInfo_apiVersion_04010: &'static [u8] =
        //     "If apiVersion is not 0, then it must be greater or equal to VK_API_VERSION_1_0"
        //         .as_bytes();
        // );

        let version = VkVersion::from_triple(Commands::VersionTriple);
        ApplicationInfo {
            inner: vk::ApplicationInfo {
                s_type: vk::StructureType::APPLICATION_INFO,
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
}

impl<'a, C> ApplicationInfo<'a, C> {
    pub const fn app_name(mut self, name: VkStr<'a>) -> Self {
        self.inner.p_application_name = name.as_ptr();
        self
    }

    pub const fn app_version(mut self, version: VkVersion) -> Self {
        self.inner.application_version = version.raw();
        self
    }

    pub const fn engine_name(mut self, name: VkStr<'a>) -> Self {
        self.inner.p_engine_name = name.as_ptr();
        self
    }

    pub const fn engine_version(mut self, version: VkVersion) -> Self {
        self.inner.engine_version = version.raw();
        self
    }
}
