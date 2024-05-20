use std::fmt;

use super::concrete_type::ScopedPhysicalDevice;

use crate::dispatchable_handles::instance::Instance;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceImageFormatProperties;

use std::mem::MaybeUninit;

impl<S, I: Instance> ScopedPhysicalDevice<S, I>
where
    I::Context: GetPhysicalDeviceImageFormatProperties,
{
    /// get image_format_properties
    ///
    /// This function takes the normal parameters via [`GetPhysicalDeviceImageFormatPropertiesParameters`], which
    /// verifies that a valid combination of parameters is used.
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html>
    #[track_caller]
    pub fn get_physical_device_image_format_properties(
        &self,
        params: GetPhysicalDeviceImageFormatPropertiesParameters,
    ) -> Result<ImageFormatProperties<S>, vk::Result> {
        let mut properties = MaybeUninit::uninit();
        let command = self
            .instance
            .context
            .GetPhysicalDeviceImageFormatProperties()
            .get_fptr();
        unsafe {
            let res = command(
                self.handle,
                params.format,
                params.image_type,
                params.image_tiling,
                params.usage_flags,
                params.create_flags,
                properties.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(ImageFormatProperties::new(properties.assume_init(), params))
        }
    }
}

/// parameters for `get_physical_device_image_format_properties`
///
/// Used to create and verify parameters for [`get_physical_device_image_format_properties`](crate::vk::SecretScope::get_physical_device_image_format_properties)
///
/// Verification checks that valid usage rules for `vkGetPhysicalDeviceImageFormatProperties`
/// are not violated, while considering that the parameters are to be **consumed by `vkCreateImage`
/// (as members of `VkImageCreateInfo`)**.
///
/// Thus, the valid usage rules for `VkImageCreateInfo` are also verified, since image format
/// properties for an image that cannot be created is meaningless, and probably undefined.
#[derive(Clone, Copy)]
pub struct GetPhysicalDeviceImageFormatPropertiesParameters {
    pub(crate) format: vk::Format,
    pub(crate) image_type: vk::ImageType,
    pub(crate) image_tiling: vk::ImageTiling,
    pub(crate) usage_flags: vk::ImageUsageFlags,
    pub(crate) create_flags: vk::ImageCreateFlags,
}

impl GetPhysicalDeviceImageFormatPropertiesParameters {
    /// create and verify parameters for [`get_physical_device_image_format_properties`](crate::vk::SecretScope::get_physical_device_image_format_properties)
    ///
    /// create and verify parameters while considering valid usage rules for
    /// `vkGetPhysicalDeviceImageFormatProperties` and `VkImageCreateInfo`.
    ///
    /// # Panic
    /// This function will panic if invalid parameters are provided.
    ///
    /// #### Why not return Result?
    /// If you provide invalid parameters, there is no useful error other than "HEY!, do it right you silly goose!".
    ///
    /// It is possible (and recommended) to create and verify the parameters in a const context
    /// to detect errors at compile time and avoid runtime checks.
    pub const fn new(
        format: vk::Format,
        image_type: vk::ImageType,
        image_tiling: vk::ImageTiling,
        usage_flags: vk::ImageUsageFlags,
        create_flags: vk::ImageCreateFlags,
    ) -> Self {
        // Verify params per the Vuids

        use vk::image_create_flag_bits::*;
        use vk::image_tiling::*;
        use vk::image_type::*;
        use vk::image_usage_flag_bits::*;

        check_vuids::check_vuids!(GetPhysicalDeviceImageFormatProperties);

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_02248: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2"
            "instead)"
            }

            assert!(!image_tiling.is(DRM_FORMAT_MODIFIER_EXT));
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_format_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "format must be a valid VkFormat value"
            }

            if !format.is_multi_planar_format() && !create_flags.contains(ALIAS_BIT) {
                assert!(
                    !create_flags.contains(DISJOINT_BIT),
                    "VUID-VkImageCreateInfo-format-01577"
                );
            }

            if create_flags.contains(SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT) {
                assert!(
                    format.has_depth_component(),
                    "VUID-VkImageCreateInfo-flags-01533"
                );
            }
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_type_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "type must be a valid VkImageType value"
            }

            if image_type.is(TYPE_1D) {
                assert!(
                    !create_flags.contains(SPARSE_RESIDENCY_BIT),
                    "VUID-VkImageCreateInfo-imageType-00970"
                );
            }

            if create_flags.contains(CUBE_COMPATIBLE_BIT) {
                assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-00949");
            }

            if create_flags.contains(TYPE_2D_ARRAY_COMPATIBLE_BIT) {
                assert!(image_type.is(TYPE_3D), "VUID-VkImageCreateInfo-flags-00950");
            }

            if create_flags.contains(SPLIT_INSTANCE_BIND_REGIONS_BIT) {
                assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02259");
                // also, mipLevels must be one, arrayLayers must be one, and imageCreateMaybeLinear (as defined in Image Creation Limits) must be VK_FALSE
            }
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "tiling must be a valid VkImageTiling value"
            }

            if image_tiling.is(LINEAR) {
                assert!(
                    !create_flags.contains(SPARSE_RESIDENCY_BIT),
                    "VUID-VkImageCreateInfo-tiling-04121"
                );
            }
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_usage_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "usage must be a valid combination of VkImageUsageFlagBits values"
            }

            if usage_flags.contains(FRAGMENT_DENSITY_MAP_BIT_EXT) {
                assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02557");
            }

            if usage_flags.contains(TRANSIENT_ATTACHMENT_BIT) {
                let must_include = COLOR_ATTACHMENT_BIT
                    .or(DEPTH_STENCIL_ATTACHMENT_BIT)
                    .or(INPUT_ATTACHMENT_BIT);
                assert!(
                    usage_flags.any_of(must_include),
                    "VUID-VkImageCreateInfo-usage-00966"
                );

                let legal_transient_flags = TRANSIENT_ATTACHMENT_BIT
                    .or(COLOR_ATTACHMENT_BIT)
                    .or(DEPTH_STENCIL_ATTACHMENT_BIT)
                    .or(INPUT_ATTACHMENT_BIT);
                assert!(
                    usage_flags.subset_of(legal_transient_flags),
                    "VUID-VkImageCreateInfo-usage-00963"
                )
            }

            if usage_flags.contains(SHADING_RATE_IMAGE_BIT_NV) {
                assert!(
                    !image_tiling.is(OPTIMAL),
                    "VUID-VkImageCreateInfo-shadingRateImage-07727"
                );
            }

            let sparse_create_flags = SPARSE_BINDING_BIT
                .or(SPARSE_RESIDENCY_BIT)
                .or(SPARSE_ALIASED_BIT);
            if create_flags.any_of(sparse_create_flags) {
                assert!(
                    !usage_flags.contains(TRANSIENT_ATTACHMENT_BIT),
                    "VUID-VkImageCreateInfo-None-01925"
                );
            }
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_usage_requiredbitmask: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "usage must not be 0"
            }

            assert!(!usage_flags.is_empty());
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be a valid combination of VkImageCreateFlagBits values"
            }

            let sparse_bits = SPARSE_ALIASED_BIT.or(SPARSE_RESIDENCY_BIT);
            if create_flags.any_of(sparse_bits) {
                assert!(
                    create_flags.contains(SPARSE_BINDING_BIT),
                    "VUID-VkImageCreateInfo-flags-00987"
                )
            }

            if create_flags.contains(BLOCK_TEXEL_VIEW_COMPATIBLE_BIT) {
                assert!(
                    format.is_compressed_format(),
                    "VUID-VkImageCreateInfo-flags-01572"
                );
                assert!(
                    create_flags.contains(MUTABLE_FORMAT_BIT),
                    "VUID-VkImageCreateInfo-flags-01573"
                );
            }

            if create_flags.contains(CORNER_SAMPLED_BIT_NV) {
                assert!(
                    image_type.is(TYPE_2D) || image_type.is(TYPE_3D),
                    "VUID-VkImageCreateInfo-flags-02050"
                );
                assert!(
                    !create_flags.contains(CUBE_COMPATIBLE_BIT)
                        && !format.has_depth_component()
                        && !format.has_stencil_component(),
                    "VUID-VkImageCreateInfo-flags-02051"
                );
                // VUID-VkImageCreateInfo-flags-02052 and VUID-VkImageCreateInfo-flags-02053 dynamic check
            }

            if create_flags.contains(SUBSAMPLED_BIT_EXT) {
                assert!(
                    image_tiling.is(OPTIMAL),
                    "VUID-VkImageCreateInfo-flags-02565"
                );
                assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02566");
                assert!(
                    !create_flags.contains(CUBE_COMPATIBLE_BIT),
                    "VUID-VkImageCreateInfo-flags-02567"
                );
                // VUID-VkImageCreateInfo-flags-02568 mim level check
            }
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceImageFormatProperties_pImageFormatProperties_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pImageFormatProperties must be a valid pointer to a VkImageFormatProperties structure"
            }

            // MaybeUninit
        }

        Self {
            format,
            image_type,
            image_tiling,
            usage_flags,
            create_flags,
        }
    }
}

// simple_struct_wrapper_scoped!(ImageFormatProperties impl Debug);

#[derive(Clone, Copy)]
pub struct ImageFormatProperties<S> {
    pub(crate) inner: vk::ImageFormatProperties,
    pub(crate) params: GetPhysicalDeviceImageFormatPropertiesParameters,
    _scope: std::marker::PhantomData<S>,
}

impl<S> ImageFormatProperties<S> {
    fn new(
        inner: vk::ImageFormatProperties,
        params: GetPhysicalDeviceImageFormatPropertiesParameters,
    ) -> Self {
        Self {
            inner,
            params,
            _scope: Default::default(),
        }
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
