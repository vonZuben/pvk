use crate::error::Error;
use crate::instance_type::{Config, InstanceType};
use crate::pretty_version::VkVersion;
use crate::type_conversions::ToC;
use crate::vk_str::VkStr;

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use vk::commands::{Commands, Extensions, LoadCommands, Version};
use vk::has_command::DestroyInstance;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateInstance.html
*/
pub fn create_instance<C: Commands>(
    create_info: &InstanceCreateInfo<C>,
) -> Result<InstanceType<Config<C>>, Error>
where
    C::Commands: DestroyInstance + Version + LoadCommands,
{
    check_vuids::check_vuids!(CreateInstance);

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_ppEnabledExtensionNames_01388: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "All required extensions for each extension in the VkInstanceCreateInfo::ppEnabledExtensionNames"
        "list must also be present in that list"
        }

        // checked in InstanceCreateInfo construction
    }

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_pCreateInfo_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "pCreateInfo must be a valid pointer to a valid VkInstanceCreateInfo structure"
        }

        // rust reference; CreateInfo validated on its own
    }

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_pAllocator_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
        "structure"
        }

        // TODO: not currently supported, always set to NULL
    }

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_pInstance_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "pInstance must be a valid pointer to a VkInstance handle"
        }

        // MaybeUninit
    }

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

impl<'a, C: Extensions> InstanceCreateInfo<'a, C> {
    pub fn new(app_info: &'a ApplicationInfo<'a, C>) -> Self {
        check_vuids::check_vuids!(InstanceCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_04925: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the pNext chain of VkInstanceCreateInfo includes a VkDebugReportCallbackCreateInfoEXT"
            "structure, the list of enabled extensions in ppEnabledExtensionNames must contain"
            "VK_EXT_debug_report"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_04926: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the pNext chain of VkInstanceCreateInfo includes a VkDebugUtilsMessengerCreateInfoEXT"
            "structure, the list of enabled extensions in ppEnabledExtensionNames must contain"
            "VK_EXT_debug_utils"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_06779: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the pNext chain includes a VkExportMetalObjectCreateInfoEXT structure, its exportObjectType"
            "member must be either VK_EXPORT_METAL_OBJECT_TYPE_METAL_DEVICE_BIT_EXT or VK_EXPORT_METAL_OBJECT_TYPE_METAL_COMMAND_QUEUE_BIT_EXT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_flags_06559: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If flags has the VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR bit set, the list"
            "of enabled extensions in ppEnabledExtensionNames must contain VK_KHR_portability_enumeration"
            }

            // TODO: flags not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the pNext chain of VkInstanceCreateInfo includes a VkDirectDriverLoadingListLUNARG"
            "structure, the list of enabled extensions in ppEnabledExtensionNames must contain"
            "VK_LUNARG_direct_driver_loading"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "sType must be VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "Each pNext member of any structure (including this one) in the pNext chain must be"
            "either NULL or a pointer to a valid instance of VkDebugReportCallbackCreateInfoEXT,"
            "VkDebugUtilsMessengerCreateInfoEXT, VkDirectDriverLoadingListLUNARG, VkExportMetalObjectCreateInfoEXT,"
            "VkValidationFeaturesEXT, or VkValidationFlagsEXT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_sType_unique: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "The sType value of each struct in the pNext chain must be unique, with the exception"
            "of structures of type VkDebugUtilsMessengerCreateInfoEXT or VkExportMetalObjectCreateInfoEXT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "flags must be a valid combination of VkInstanceCreateFlagBits values"
            }

            // TODO: flags not currently supported (always empty)
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pApplicationInfo_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If pApplicationInfo is not NULL, pApplicationInfo must be a valid pointer to a valid"
            "VkApplicationInfo structure"
            }

            // rust reference; ApplicationInfo self validated
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_ppEnabledLayerNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If enabledLayerCount is not 0, ppEnabledLayerNames must be a valid pointer to an array"
            "of enabledLayerCount null-terminated UTF-8 strings"
            }

            // TODO: layers not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_ppEnabledExtensionNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If enabledExtensionCount is not 0, ppEnabledExtensionNames must be a valid pointer"
            "to an array of enabledExtensionCount null-terminated UTF-8 strings"
            }

            // TODO: extensions not currently supported
        }

        let extensions = C::list_of_extensions();
        let extensions = extensions.as_ref();

        Self {
            inner: vk::InstanceCreateInfo {
                s_type: vk::StructureType::INSTANCE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::InstanceCreateFlags::empty(),
                p_application_info: &app_info.inner,
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: extensions
                    .len()
                    .try_into()
                    .expect("list of extensions len bigger than u32::MAX"),
                pp_enabled_extension_names: extensions.as_ptr(),
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
    pub const fn new<C: Copy>(_: C) -> ApplicationInfo<'a, C>
    where
        C: Commands,
        C::Commands: Version,
    {
        check_vuids::check_vuids!(ApplicationInfo);

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_apiVersion_04010: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If apiVersion is not 0, then it must be greater than or equal to VK_API_VERSION_1_0"
            }

            // Version trait will provide a proper version
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "sType must be VK_STRUCTURE_TYPE_APPLICATION_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pNext must be NULL"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_pApplicationName_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If pApplicationName is not NULL, pApplicationName must be a null-terminated UTF-8"
            "string"
            }

            // ensured by VkStr
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_pEngineName_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If pEngineName is not NULL, pEngineName must be a null-terminated UTF-8 string"
            }

            // ensured by VkStr
        }

        let version = VkVersion::from_triple(C::Commands::VERSION_TRIPLE);
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
