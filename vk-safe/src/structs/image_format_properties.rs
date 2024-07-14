use std::fmt;

use super::ImageParameters;

use vk_safe_sys as vk;

/// Structure specifying an image format properties
///
/// The version of this struct in this crate also hold the
/// [`ImageParameters`] that were used to obtain it for performing
/// checks in other parts of the api.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageFormatProperties.html>
#[derive(Clone, Copy)]
pub struct ImageFormatProperties<S> {
    inner: vk::ImageFormatProperties,
    params: ImageParameters,
    _scope: std::marker::PhantomData<S>,
}

impl<S> ImageFormatProperties<S> {
    /// Make new ImageFormatProperties
    ///
    /// This is unsafe because you must only create it with
    /// [`vk::ImageFormatProperties`] that were obtained with the same [`ImageParameters`]
    pub(crate) unsafe fn new(inner: vk::ImageFormatProperties, params: ImageParameters) -> Self {
        Self {
            inner,
            params,
            _scope: Default::default(),
        }
    }

    /// Get a reference to the [`ImageParameters`] used to obtain these image properties
    pub fn image_parameters(&self) -> &ImageParameters {
        &self.params
    }
}

impl<S> std::fmt::Debug for ImageFormatProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<S> std::ops::Deref for ImageFormatProperties<S> {
    type Target = vk::ImageFormatProperties;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
