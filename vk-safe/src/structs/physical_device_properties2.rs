use crate::vk::PhysicalDeviceProperties;

use crate::type_conversions::ConvertWrapper;

struct_wrapper!(
/// VkPhysicalDeviceProperties2 - Structure specifying physical device properties
///
/// <https://registry.khronos.org/VulkanSC/specs/1.0-extensions/man/html/VkPhysicalDeviceProperties2.html>
PhysicalDeviceProperties2<Pd,>
);

impl<Pd> std::ops::Deref for PhysicalDeviceProperties2<Pd> {
    type Target = PhysicalDeviceProperties<Pd>;

    fn deref(&self) -> &Self::Target {
        unsafe { ConvertWrapper::from_c(&self.inner.properties) }
    }
}

impl<Pd> std::fmt::Debug for PhysicalDeviceProperties2<Pd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}
