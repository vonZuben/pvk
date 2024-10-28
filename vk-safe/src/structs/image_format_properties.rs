struct_wrapper!(
/// Structure specifying an image format properties
///
/// This struct is generic over the [`ImageParameters`](crate::vk::ImageParameters) that
/// were used to obtain it.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageFormatProperties.html>
ImageFormatProperties<S, Params,>
impl Clone, Copy, Debug, Deref
);
