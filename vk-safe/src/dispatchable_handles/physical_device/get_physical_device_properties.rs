/*!
Query the properties of the PhysicalDevice

use the [`get_physical_device_properties`](ScopedPhysicalDevice::get_physical_device_properties) method on a scoped PhysicalDevice

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html>
*/

use super::concrete_type::ScopedPhysicalDevice;
use super::PhysicalDeviceConfig;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceProperties;

use std::fmt;
use std::mem::MaybeUninit;

impl<S, C: PhysicalDeviceConfig> ScopedPhysicalDevice<S, C>
where
    C::Context: GetPhysicalDeviceProperties,
{
    /**
    Query the properties of the PhysicalDevice

    ```rust
    # use vk_safe::vk;
    # vk::device_context!(D: VERSION_1_0);
    # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    #   (physical_device: P) {
    let physical_device_properties = physical_device.get_physical_device_properties();
    # }
    ```
    */
    pub fn get_physical_device_properties(&self) -> PhysicalDeviceProperties<S> {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance()
                .context
                .GetPhysicalDeviceProperties()
                .get_fptr()(self.handle, properties.as_mut_ptr());
            PhysicalDeviceProperties::new(properties.assume_init())
        }
    }
}

const _VUID: () = {
    check_vuids::check_vuids!(GetPhysicalDeviceProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceProperties_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pProperties must be a valid pointer to a VkPhysicalDeviceProperties structure"
        }

        // MaybeUninit
    }
};

simple_struct_wrapper_scoped!(PhysicalDeviceProperties impl Deref);

#[repr(transparent)]
/// Universally unique identifier for a PhysicalDevice
pub struct PipelineCacheUUID {
    uuid: [u8; vk::UUID_SIZE],
}

impl std::fmt::Debug for PipelineCacheUUID {
    /// print the UUID
    ///
    /// outputs the UUID with each byte printed in upper hex and 0 padded
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PipelineCacheUUID[")?;
        for b in self.uuid {
            write!(f, "{:0<2X}", b)?;
        }
        write!(f, "]")
    }
}

impl<S> PhysicalDeviceProperties<S> {
    pretty_version!(api_version);
    get_str!(device_name);

    /// Get the `PipelineCacheUUID` for this PhysicalDevice
    pub fn pipeline_cache_uuid(&self) -> &PipelineCacheUUID {
        unsafe {
            std::mem::transmute::<&[u8; vk::UUID_SIZE], &PipelineCacheUUID>(
                &self.pipeline_cache_uuid,
            )
        }
    }
}

impl<S> fmt::Debug for PhysicalDeviceProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceProperties")
            .field("api_version", &self.api_version())
            .field("driver_version", &self.driver_version)
            .field("vender_id", &self.vendor_id)
            .field("device_id", &self.device_id)
            .field("device_type", &self.device_type)
            .field("device_name", &self.device_name())
            .field("pipeline_cache_uuid", &self.pipeline_cache_uuid())
            .field("limits", &self.limits)
            .field("sparse_properties", &self.sparse_properties)
            .finish()
    }
}
