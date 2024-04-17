use super::*;
use crate::instance_type::Instance;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceProperties;

use std::fmt;
use std::mem::MaybeUninit;

impl<S, I: Instance> ScopedPhysicalDeviceType<S, I>
where
    I::Context: GetPhysicalDeviceProperties,
{
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html>
    pub fn get_physical_device_properties(&self) -> PhysicalDeviceProperties<S> {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance
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

impl<S> PhysicalDeviceProperties<S> {
    pretty_version!(api_version);
    get_str!(device_name);
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
            .field("pipeline_cache_id", &self.pipeline_cache_uuid)
            .field("limits", &self.limits)
            .field("sparse_properties", &self.sparse_properties)
            .finish()
    }
}
