use crate::type_conversions::ConvertWrapper;

struct_wrapper!(
/// Structure describing the fine-grained features that can be supported by an implementation
///
/// See Vulkan Docs for details about the different features
/// <https://registry.khronos.org/vulkan/specs/latest/man/html/VkPhysicalDeviceFeatures2.html>
PhysicalDeviceFeatures2<S,>);

impl<S> std::ops::Deref for PhysicalDeviceFeatures2<S> {
    type Target = crate::vk::PhysicalDeviceFeatures<S>;

    fn deref(&self) -> &Self::Target {
        unsafe { ConvertWrapper::from_c(&self.inner.features) }
    }
}

impl<S> std::fmt::Debug for PhysicalDeviceFeatures2<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.features.fmt(f)
    }
}
