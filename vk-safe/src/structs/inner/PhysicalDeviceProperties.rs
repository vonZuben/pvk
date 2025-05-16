use std::fmt;

use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

struct_wrapper!(
/// Structure specifying physical device properties
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceProperties.html>
PhysicalDeviceProperties<S,> impl Deref);

#[repr(transparent)]
/// Universally unique identifier for a PhysicalDevice
pub struct PipelineCacheUUID {
    uuid: [u8; vk::UUID_SIZE],
}

unsafe impl ConvertWrapper<[u8; vk::UUID_SIZE]> for PipelineCacheUUID {}

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
        unsafe { <&PipelineCacheUUID>::from_c(&self.pipeline_cache_uuid) }
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
