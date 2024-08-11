#[allow(non_snake_case)]
pub mod ImageParameters {
    use std::fmt;
    use std::marker::PhantomData;

    use vk_safe_sys as vk;

    use vk::enum_traits::*;
    use vk::flag_traits::*;

    pub unsafe trait ImageParameters: Send + Sync + Copy + Clone + fmt::Debug {
        type Format: Format;
        type ImageType: ImageType;
        type ImageTiling: ImageTiling;
        type ImageUsageFlags: ImageUsageFlags;
        type ImageCreateFlags: ImageCreateFlags;

        fn format() -> vk::Format {
            Self::Format::VALUE
        }

        fn image_type() -> vk::ImageType {
            Self::ImageType::VALUE
        }

        fn image_tiling() -> vk::ImageTiling {
            Self::ImageTiling::VALUE
        }

        fn image_usage_flags() -> vk::ImageUsageFlags {
            Self::ImageUsageFlags::INCLUDES
        }

        fn image_create_flags() -> vk::ImageCreateFlags {
            Self::ImageCreateFlags::INCLUDES
        }
    }

    /// create and verify image parameters
    ///
    /// # Panic
    /// This function will panic if invalid parameters are provided.
    ///
    /// #### Why not return Result?
    /// If you provide invalid parameters, there is no useful error other than "HEY!, do it right you silly goose!".
    ///
    /// It is possible (and recommended) to create and verify the parameters in a const context
    /// to detect errors at compile time and avoid runtime checks.
    pub fn new<
        Format: vk::enum_traits::Format,
        ImageType: vk::enum_traits::ImageType,
        ImageTiling: vk::enum_traits::ImageTiling,
        ImageUsageFlags: vk::flag_traits::ImageUsageFlags,
        ImageCreateFlags: vk::flag_traits::ImageCreateFlags,
    >(
        format: Format,
        image_type: ImageType,
        image_tiling: ImageTiling,
        usage_flags: ImageUsageFlags,
        create_flags: ImageCreateFlags,
    ) -> impl ImageParameters {
        let _ = (format, image_type, image_tiling, usage_flags, create_flags);

        const {
            use vk::image_create_flag_bits::*;
            use vk::image_tiling::*;
            use vk::image_type::*;
            use vk::image_usage_flag_bits::*;

            let format = Format::VALUE;
            let image_type = ImageType::VALUE;
            let image_tiling = ImageTiling::VALUE;
            let usage_flags = ImageUsageFlags::INCLUDES;
            let create_flags = ImageCreateFlags::INCLUDES;

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

            _ImageParameters {
                parameters: PhantomData::<(
                    Format,
                    ImageType,
                    ImageTiling,
                    ImageUsageFlags,
                    ImageCreateFlags,
                )>,
            }
        }
    }

    /// Non-standard struct for parameters of an Image
    ///
    /// Used to create and verify parameters for an Image that can be used
    /// in various Vulkan Commands
    ///
    /// Verification checks that valid usage rules for `vkGetPhysicalDeviceImageFormatProperties`
    /// are not violated, while considering that the parameters are to be **consumed by `vkCreateImage`
    /// (as members of `VkImageCreateInfo`)**.
    ///
    /// Thus, the valid usage rules for `VkImageCreateInfo` are also verified, since image format
    /// properties for an image that cannot be created is meaningless, and probably undefined.
    ///
    /// 🚧 Other rules need to be checked?
    struct _ImageParameters<Parameters> {
        parameters: PhantomData<Parameters>,
    }

    impl<P> Clone for _ImageParameters<P> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<P> Copy for _ImageParameters<P> {}

    unsafe impl<P> Send for _ImageParameters<P> {}
    unsafe impl<P> Sync for _ImageParameters<P> {}

    unsafe impl<
            Format: vk::enum_traits::Format,
            ImageType: vk::enum_traits::ImageType,
            ImageTiling: vk::enum_traits::ImageTiling,
            ImageUsageFlags: vk::flag_traits::ImageUsageFlags,
            ImageCreateFlags: vk::flag_traits::ImageCreateFlags,
        > ImageParameters
        for _ImageParameters<(
            Format,
            ImageType,
            ImageTiling,
            ImageUsageFlags,
            ImageCreateFlags,
        )>
    {
        type Format = Format;
        type ImageType = ImageType;
        type ImageTiling = ImageTiling;
        type ImageUsageFlags = ImageUsageFlags;
        type ImageCreateFlags = ImageCreateFlags;
    }

    impl<Parameters> fmt::Debug for _ImageParameters<Parameters>
    where
        Self: ImageParameters,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("ImageParameters")
                .field("format", &<Self as ImageParameters>::Format::VALUE)
                .field("image_type", &<Self as ImageParameters>::ImageType::VALUE)
                .field(
                    "image_tiling",
                    &<Self as ImageParameters>::ImageTiling::VALUE,
                )
                .field(
                    "usage_flags",
                    &<Self as ImageParameters>::ImageUsageFlags::INCLUDES,
                )
                .field(
                    "create_flags",
                    &<Self as ImageParameters>::ImageCreateFlags::INCLUDES,
                )
                .finish()
        }
    }
}
