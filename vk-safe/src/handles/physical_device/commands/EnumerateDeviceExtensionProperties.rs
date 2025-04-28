use crate::enumerator::Enumerator;
use crate::handles::DispatchableHandle;
use crate::scope::Captures;
use crate::structs::ExtensionProperties;
use crate::VkStr;

use vk_safe_sys as vk;

pub trait EnumerateDeviceExtensionProperties:
    DispatchableHandle<RawHandle = vk::PhysicalDevice>
{
    /// Query the device level extensions supported by the PhysicalDevice
    ///
    /// If `layer_name` is `None`, only extensions provided by the Vulkan implementation.
    /// are returned. If `layer_name` is `Some(layer_name)`, device extensions provided
    /// by that layer are returned.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let extension_properties =
    ///     physical_device.enumerate_device_extension_properties(None).auto_get_enumerate();
    /// # }
    /// ```
    fn enumerate_device_extension_properties<'p, 'a, X>(
        &'p self,
        layer_name: Option<VkStr<'a>>,
    ) -> impl Enumerator<ExtensionProperties<Self>> + Captures<(&'p Self, VkStr<'a>)>
    where
        Self::Commands: vk::has_command::EnumerateDeviceExtensionProperties<X>,
    {
        use vk::has_command::EnumerateDeviceExtensionProperties;

        check_vuids::check_vuids!(EnumerateDeviceExtensionProperties);

        #[allow(unused_labels)]
        'VUID_vkEnumerateDeviceExtensionProperties_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // always valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkEnumerateDeviceExtensionProperties_pLayerName_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pLayerName is not NULL, pLayerName must be a null-terminated UTF-8 string"
            }

            // Option<VkStr>
        }

        #[allow(unused_labels)]
        'VUID_vkEnumerateDeviceExtensionProperties_pPropertyCount_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pPropertyCount must be a valid pointer to a uint32_t value"
            }

            // enumerator_code2!
        }

        #[allow(unused_labels)]
        'VUID_vkEnumerateDeviceExtensionProperties_pProperties_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties"
            "must be a valid pointer to an array of pPropertyCount VkExtensionProperties structures"
            }

            // enumerator_code2!
        }

        make_enumerator!(
            self.commands().EnumerateDeviceExtensionProperties().get_fptr();
            (self.raw_handle(), layer_name)
        )
    }
}
