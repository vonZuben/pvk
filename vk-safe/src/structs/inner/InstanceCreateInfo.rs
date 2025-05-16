use std::marker::PhantomData;

use super::ApplicationInfo;

use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::context::Extensions;

/// Info for creating an instance
///
/// Those familiar with Vulkan will know that this is where you normally indicate what Extensions and Layers you want to use. In vk-safe
/// Extensions are indicated with [vk::instance_context] and passed into [ApplicationInfo]. Layers are not currently supported, but are planned.
///
/// Currently, the only thing you need to create this structure is an instance of [ApplicationInfo].
/// In future, support will be added for using p_next for additional functionality.
///
/// See also
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkInstanceCreateInfo.html>
pub struct InstanceCreateInfo<'a, C> {
    pub(crate) inner: vk::InstanceCreateInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
}

impl<'a, C: Extensions> InstanceCreateInfo<'a, C> {
    /// Create InstanceCreateInfo from [ApplicationInfo]
    pub fn new(app_info: &'a ApplicationInfo<'a, C>) -> Self {
        check_vuids::check_vuids!(InstanceCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_04925: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain of VkInstanceCreateInfo includes a VkDebugReportCallbackCreateInfoEXT"
            "structure, the list of enabled extensions in ppEnabledExtensionNames must contain"
            "VK_EXT_debug_report"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_04926: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain of VkInstanceCreateInfo includes a VkDebugUtilsMessengerCreateInfoEXT"
            "structure, the list of enabled extensions in ppEnabledExtensionNames must contain"
            "VK_EXT_debug_utils"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_06779: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkExportMetalObjectCreateInfoEXT structure, its exportObjectType"
            "member must be either VK_EXPORT_METAL_OBJECT_TYPE_METAL_DEVICE_BIT_EXT or VK_EXPORT_METAL_OBJECT_TYPE_METAL_COMMAND_QUEUE_BIT_EXT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_flags_06559: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If flags has the VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR bit set, the list"
            "of enabled extensions in ppEnabledExtensionNames must contain VK_KHR_portability_enumeration"
            }

            // TODO: flags not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain of VkInstanceCreateInfo includes a VkDirectDriverLoadingListLUNARG"
            "structure, the list of enabled extensions in ppEnabledExtensionNames must contain"
            "VK_LUNARG_direct_driver_loading"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
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
            check_vuids::description! {
            "The sType value of each struct in the pNext chain must be unique, with the exception"
            "of structures of type VkDebugUtilsMessengerCreateInfoEXT or VkExportMetalObjectCreateInfoEXT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be a valid combination of VkInstanceCreateFlagBits values"
            }

            // TODO: flags not currently supported (always empty)
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_pApplicationInfo_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pApplicationInfo is not NULL, pApplicationInfo must be a valid pointer to a valid"
            "VkApplicationInfo structure"
            }

            // rust reference; ApplicationInfo self validated
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_ppEnabledLayerNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If enabledLayerCount is not 0, ppEnabledLayerNames must be a valid pointer to an array"
            "of enabledLayerCount null-terminated UTF-8 strings"
            }

            // TODO: layers not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkInstanceCreateInfo_ppEnabledExtensionNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If enabledExtensionCount is not 0, ppEnabledExtensionNames must be a valid pointer"
            "to an array of enabledExtensionCount null-terminated UTF-8 strings"
            }

            // a proper implementation of the unsafe Extensions trait ensures this
        }

        let extensions = C::list_of_extensions();
        let extensions = extensions.as_ref();

        Self {
            inner: vk::InstanceCreateInfo {
                s_type: vk::StructureType::INSTANCE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::InstanceCreateFlags::empty(),
                p_application_info: app_info.to_c(),
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: extensions
                    .len()
                    .try_into()
                    .expect("list of extensions len bigger than u32::MAX"),
                pp_enabled_extension_names: extensions.as_ptr().cast(),
            },
            _config: PhantomData,
            _refs: PhantomData,
        }
    }
}
