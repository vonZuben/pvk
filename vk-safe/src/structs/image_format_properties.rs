use std::fmt;

use vk_safe_sys as vk;

/// Structure specifying an image format properties
///
/// This struct is generic over the [`ImageParameters`](crate::vk::ImageParameters) that
/// were used to obtain it.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageFormatProperties.html>
#[derive(Clone, Copy)]
pub struct ImageFormatProperties<S, Params> {
    inner: vk::ImageFormatProperties,
    scope: std::marker::PhantomData<S>,
    image_params: std::marker::PhantomData<Params>,
}

impl<S, Params> ImageFormatProperties<S, Params> {
    /// Make new ImageFormatProperties
    ///
    /// This is unsafe because you must only create it with
    /// [`vk::ImageFormatProperties`] that were obtained with the
    /// same [`ImageParameters`](crate::vk::ImageParameters).
    pub(crate) unsafe fn new(inner: vk::ImageFormatProperties, _params: Params) -> Self {
        Self {
            inner,
            scope: std::marker::PhantomData,
            image_params: std::marker::PhantomData,
        }
    }
}

impl<S, Params> std::fmt::Debug for ImageFormatProperties<S, Params> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<S, Params> std::ops::Deref for ImageFormatProperties<S, Params> {
    type Target = vk::ImageFormatProperties;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
