use super::*;
use crate::instance::InstanceConfig;
use vk::GetCommand;
use vk_safe_sys as vk;

use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html
*/
impl<'scope, C: InstanceConfig> ScopedPhysicalDevice<'scope, '_, C>
where
    C::Commands: GetCommand<vk::GetPhysicalDeviceImageFormatProperties>,
{
    #[track_caller]
    pub fn get_physical_device_image_format_properties(
        &self,
        params: GetPhysicalDeviceImageFormatPropertiesParams,
    ) -> Result<ImageFormatProperties<'scope>, vk::Result> {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.commands.get_command().get_fptr()(
                self.handle,
                params.format,
                params.image_type,
                params.image_tiling,
                params.usage_flags,
                params.create_flags,
                properties.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(ImageFormatProperties::new(properties.assume_init()))
        }
    }
}

pub struct GetPhysicalDeviceImageFormatPropertiesParams {
    format: vk::Format,
    image_type: vk::ImageType,
    image_tiling: vk::ImageTiling,
    usage_flags: vk::ImageUsageFlags,
    create_flags: vk::ImageCreateFlags,
}

impl GetPhysicalDeviceImageFormatPropertiesParams {
    pub const fn new(format: vk::Format, image_type: vk::ImageType, image_tiling: vk::ImageTiling, usage_flags: vk::ImageUsageFlags, create_flags: vk::ImageCreateFlags) -> Self {
        // Verify params per the Vuids

        use vk::image_usage_flag_bits::*;
        use vk::image_create_flag_bits::*;
        use vk::image_tiling::*;
        use vk::image_type::*;

        check_vuid_defs2!{ GetPhysicalDeviceImageFormatProperties
            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_02248 : & 'static [ u8 ] = "tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2 instead)." . as_bytes ( ) ;
            CHECK {
                assert!(!image_tiling.is(DRM_FORMAT_MODIFIER_EXT));
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_physicalDevice_parameter:
                &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
            CHECK {
                // ensured by PhysicalDevice creation
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_format_parameter: &'static [u8] =
                "format must be a valid VkFormat value".as_bytes();
            CHECK {
                if !format.is_multi_planar_format() && !create_flags.contains(ALIAS_BIT) {
                    assert!(!create_flags.contains(DISJOINT_BIT), "VUID-VkImageCreateInfo-format-01577");
                }

                if create_flags.contains(SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT) {
                    assert!(format.has_depth_component(), "VUID-VkImageCreateInfo-flags-01533");
                }
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_type_parameter: &'static [u8] =
                "type must be a valid VkImageType value".as_bytes();
            CHECK {
                if image_type.is(TYPE_1D) {
                    assert!(!create_flags.contains(SPARSE_RESIDENCY_BIT), "VUID-VkImageCreateInfo-imageType-00970");
                }

                if create_flags.contains(CUBE_COMPATIBLE_BIT) {
                    assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-00949");
                }

                if create_flags.contains(TYPE_2D_ARRAY_COMPATIBLE_BIT) {
                    assert!(image_type.is(TYPE_3D), "VUID-VkImageCreateInfo-flags-00950");
                }

                if create_flags.contains(SPLIT_INSTANCE_BIND_REGIONS_BIT) {
                    assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02259"); // also, mipLevels must be one, arrayLayers must be one, and imageCreateMaybeLinear (as defined in Image Creation Limits) must be VK_FALSE
                }
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_parameter: &'static [u8] =
                "tiling must be a valid VkImageTiling value".as_bytes();
            CHECK {
                if image_tiling.is(LINEAR) {
                    assert!(!create_flags.contains(SPARSE_RESIDENCY_BIT), "VUID-VkImageCreateInfo-tiling-04121");
                }
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_usage_parameter: &'static [u8] =
                "usage must be a valid combination of VkImageUsageFlagBits values".as_bytes();
            CHECK {
                if usage_flags.contains(FRAGMENT_DENSITY_MAP_BIT_EXT) {
                    assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02557");
                }

                if usage_flags.contains(TRANSIENT_ATTACHMENT_BIT) {
                    let must_include = COLOR_ATTACHMENT_BIT.or(DEPTH_STENCIL_ATTACHMENT_BIT).or(INPUT_ATTACHMENT_BIT);
                    assert!(usage_flags.any_of(must_include), "VUID-VkImageCreateInfo-usage-00966");

                    let legal_transient_flags = TRANSIENT_ATTACHMENT_BIT.or(COLOR_ATTACHMENT_BIT).or(DEPTH_STENCIL_ATTACHMENT_BIT).or(INPUT_ATTACHMENT_BIT);
                    assert!(usage_flags.subset_of(legal_transient_flags), "VUID-VkImageCreateInfo-usage-00963")
                }

                if usage_flags.contains(SHADING_RATE_IMAGE_BIT_NV) {
                    assert!(!image_tiling.is(OPTIMAL), "VUID-VkImageCreateInfo-shadingRateImage-07727");
                }

                let sparse_create_flags = SPARSE_BINDING_BIT.or(SPARSE_RESIDENCY_BIT).or(SPARSE_ALIASED_BIT);
                if create_flags.any_of(sparse_create_flags) {
                    assert!(!usage_flags.contains(TRANSIENT_ATTACHMENT_BIT), "VUID-VkImageCreateInfo-None-01925");
                }
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_usage_requiredbitmask:
                &'static [u8] = "usage must not be 0".as_bytes();
            CHECK {
                assert!(!usage_flags.is_empty());
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_flags_parameter: &'static [u8] =
                "flags must be a valid combination of VkImageCreateFlagBits values".as_bytes();
            CHECK {
                let sparse_bits = SPARSE_ALIASED_BIT.or(SPARSE_RESIDENCY_BIT);
                if create_flags.any_of(sparse_bits) {
                    assert!(create_flags.contains(SPARSE_BINDING_BIT), "VUID-VkImageCreateInfo-flags-00987")
                }

                if create_flags.contains(BLOCK_TEXEL_VIEW_COMPATIBLE_BIT) {
                    assert!(format.is_compressed_format(), "VUID-VkImageCreateInfo-flags-01572");
                    assert!(create_flags.contains(MUTABLE_FORMAT_BIT), "VUID-VkImageCreateInfo-flags-01573");
                }

                if create_flags.contains(CORNER_SAMPLED_BIT_NV) {
                    assert!(image_type.is(TYPE_2D) || image_type.is(TYPE_3D), "VUID-VkImageCreateInfo-flags-02050");
                    assert!(!create_flags.contains(CUBE_COMPATIBLE_BIT) && !format.has_depth_component() && !format.has_stencil_component(), "VUID-VkImageCreateInfo-flags-02051");
                    // VUID-VkImageCreateInfo-flags-02052 and VUID-VkImageCreateInfo-flags-02053 dynamic check
                }

                if create_flags.contains(SUBSAMPLED_BIT_EXT) {
                    assert!(image_tiling.is(OPTIMAL), "VUID-VkImageCreateInfo-flags-02565");
                    assert!(image_type.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02566");
                    assert!(!create_flags.contains(CUBE_COMPATIBLE_BIT), "VUID-VkImageCreateInfo-flags-02567");
                    // VUID-VkImageCreateInfo-flags-02568 mim level check
                }
            }

            pub const VUID_vkGetPhysicalDeviceImageFormatProperties_pImageFormatProperties_parameter:
                &'static [u8] =
                "pImageFormatProperties must be a valid pointer to a VkImageFormatProperties structure"
                    .as_bytes();
            CHECK {
                // MaybeUninit provides
            }
        };

        Self { format, image_type, image_tiling, usage_flags, create_flags}
    }
}

simple_struct_wrapper_scoped!(ImageFormatProperties impl Debug);