/*!
Query the device level extensions supported by the PhysicalDevice

Returns properties of the extensions that are available on the PhysicalDevice. Some extensions may
be provided by a layer, which can be queried by providing the layer name.

use the [`enumerate_device_extension_properties`](ScopedPhysicalDevice::enumerate_device_extension_properties) method on a scoped PhysicalDevice

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceExtensionProperties.html>
*/

use super::concrete_type::ScopedPhysicalDevice;
use super::PhysicalDeviceConfig;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::vk_str::VkStr;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceExtensionProperties;

pub use crate::dispatchable_handles::common::extension_properties::ExtensionProperties;

impl<S, C: PhysicalDeviceConfig> ScopedPhysicalDevice<S, C>
where
    C::Context: EnumerateDeviceExtensionProperties,
{
    /**
    Query the device level extensions supported by the PhysicalDevice

    If `layer_name` is `None`, only extensions provided by the Vulkan implementation. If `layer_name`
    is `Some(layer_name)`, device extensions provided by that layer are returned.

    Must also provide the storage space to return the extension properties.

    ```rust
    # use vk_safe::vk;
    # vk::device_context!(D: VERSION_1_0);
    # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    #   (physical_device: P) {
    let extension_properties = physical_device.enumerate_device_extension_properties(None, Vec::new());
    # }
    ```
    */
    pub fn enumerate_device_extension_properties<A: ArrayStorage<ExtensionProperties<S>>>(
        &self,
        layer_name: Option<VkStr>,
        mut storage: A,
    ) -> Result<A::InitStorage, Error> {
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

        enumerator_code2!(self.instance().context.EnumerateDeviceExtensionProperties().get_fptr(); (self.handle, layer_name) -> storage)
    }
}
