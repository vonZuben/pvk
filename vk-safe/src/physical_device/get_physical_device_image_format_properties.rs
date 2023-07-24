use super::*;
use crate::instance::InstanceConfig;
use vk::GetCommand;
use vk_safe_sys as vk;

use std::fmt;
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
        format: impl vk::FormatConst,
        image_type: impl vk::ImageTypeConst,
        image_tiling: impl vk::ImageTilingConst,
        usage_flags: impl vk::ImageUsageFlagsConst,
        create_flags: impl vk::ImageCreateFlagsConst,//vk::ImageCreateFlags,
    ) -> Result<ImageFormatProperties<'scope>, vk::Result> {
        get_physical_device_image_format_properties_validation::Validation::verify(image_tiling, usage_flags, create_flags, image_type, format);
        let mut properties = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.commands.get_command().get_fptr()(
                self.handle,
                format.variant(),
                image_type.variant(),
                image_tiling.variant(),
                usage_flags.bitmask(),
                create_flags.bitmask(),
                properties.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(ImageFormatProperties::new(properties.assume_init()))
        }
    }
}

mod get_physical_device_image_format_properties_validation {
    use vk_safe_sys::validation::GetPhysicalDeviceImageFormatProperties::*;

    use vk_safe_sys as vk;
    use vk::image_usage_flag_bits::*;
    use vk::image_create_flag_bits::*;
    use vk::image_tiling::*;
    use vk::image_type::*;

    verify_vuids!(
        pub Validation(
            Tiling: vk::ImageTilingConst,
            Usage: vk::ImageUsageFlagsConst,
            Create: vk::ImageCreateFlagsConst,
            Type: vk::ImageTypeConst,
            Format: vk::FormatConst
        ) {
            const VUID_vkGetPhysicalDeviceImageFormatProperties_flags_parameter: () = {
                let image_tiling = vk::raw_enum_from_type!(Tiling);
                // let usage_flags = vk::raw_bitmask_from_type!(Usage);
                let create_flags = vk::raw_bitmask_from_type!(Create);
                let image_ty = vk::raw_enum_from_type!(Type);

                let sparse_bits = bitmask!(SPARSE_ALIASED_BIT | SPARSE_RESIDENCY_BIT);
                if create_flags.any_of(sparse_bits) {
                    assert!(create_flags.contains(SPARSE_BINDING_BIT), "VUID-VkImageCreateInfo-flags-00987")
                }

                if create_flags.contains(BLOCK_TEXEL_VIEW_COMPATIBLE_BIT) {
                    assert!(Format::COMPRESSED_FORMAT, "VUID-VkImageCreateInfo-flags-01572");
                    assert!(create_flags.contains(MUTABLE_FORMAT_BIT), "VUID-VkImageCreateInfo-flags-01573");
                }

                if create_flags.contains(CORNER_SAMPLED_BIT_NV) {
                    assert!(image_ty.is(TYPE_2D) || image_ty.is(TYPE_3D), "VUID-VkImageCreateInfo-flags-02050");
                    assert!(!create_flags.contains(CUBE_COMPATIBLE_BIT) && !Format::HAS_DEPTH_COMPONENT && !Format::HAS_STENCIL_COMPONENT, "VUID-VkImageCreateInfo-flags-02051");
                    // VUID-VkImageCreateInfo-flags-02052 and VUID-VkImageCreateInfo-flags-02053 dynamic check
                }

                if create_flags.contains(SUBSAMPLED_BIT_EXT) {
                    assert!(image_tiling.is(OPTIMAL), "VUID-VkImageCreateInfo-flags-02565");
                    assert!(image_ty.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02566");
                    assert!(!create_flags.contains(CUBE_COMPATIBLE_BIT), "VUID-VkImageCreateInfo-flags-02567");
                    // VUID-VkImageCreateInfo-flags-02568 mim level check
                }
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_format_parameter: () = {
                // let image_tiling = vk::raw_enum_from_type!(Tiling);
                // let usage_flags = vk::raw_bitmask_from_type!(Usage);
                let create_flags = vk::raw_bitmask_from_type!(Create);
                // let image_ty = vk::raw_enum_from_type!(Type);

                if !Format::MULTI_PLANAR && !create_flags.contains(ALIAS_BIT) {
                    assert!(!create_flags.contains(DISJOINT_BIT), "VUID-VkImageCreateInfo-format-01577")
                }

                if create_flags.contains(SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT) {
                    assert!(Format::HAS_DEPTH_COMPONENT, "VUID-VkImageCreateInfo-flags-01533")
                }
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_pImageFormatProperties_parameter: () = {
                // suing MaybeUninit
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_physicalDevice_parameter: () = {
                // ensured by PhysicalDevice creation
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_02248: () = {
                let image_tiling = vk::raw_enum_from_type!(Tiling);
                assert!(!image_tiling.is(DRM_FORMAT_MODIFIER_EXT));
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_parameter: () = {
                let image_tiling = vk::raw_enum_from_type!(Tiling);
                // let usage_flags = vk::raw_bitmask_from_type!(Usage);
                let create_flags = vk::raw_bitmask_from_type!(Create);
                // let image_ty = vk::raw_enum_from_type!(Type);

                if image_tiling.is(LINEAR) {
                    assert!(!create_flags.contains(SPARSE_RESIDENCY_BIT), "VUID-VkImageCreateInfo-tiling-04121")
                }
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_type_parameter: () = {
                // let image_tiling = vk::raw_enum_from_type!(Tiling);
                // let usage_flags = vk::raw_bitmask_from_type!(Usage);
                let create_flags = vk::raw_bitmask_from_type!(Create);
                let image_ty = vk::raw_enum_from_type!(Type);

                if image_ty.is(TYPE_1D) {
                    assert!(!create_flags.contains(SPARSE_RESIDENCY_BIT), "VUID-VkImageCreateInfo-imageType-00970")
                }

                if create_flags.contains(CUBE_COMPATIBLE_BIT) {
                    assert!(image_ty.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-00949")
                }

                if create_flags.contains(TYPE_2D_ARRAY_COMPATIBLE_BIT) {
                    assert!(image_ty.is(TYPE_3D), "VUID-VkImageCreateInfo-flags-00950")
                }

                if create_flags.contains(SPLIT_INSTANCE_BIND_REGIONS_BIT) {
                    assert!(image_ty.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02259") // also, mipLevels must be one, arrayLayers must be one, and imageCreateMaybeLinear (as defined in Image Creation Limits) must be VK_FALSE
                }
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_usage_parameter: () = {
                let image_tiling = vk::raw_enum_from_type!(Tiling);
                let usage_flags = vk::raw_bitmask_from_type!(Usage);
                let create_flags = vk::raw_bitmask_from_type!(Create);
                let image_ty = vk::raw_enum_from_type!(Type);

                if usage_flags.contains(FRAGMENT_DENSITY_MAP_BIT_EXT) {
                    assert!(image_ty.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02557")
                }

                if usage_flags.contains(TRANSIENT_ATTACHMENT_BIT) {
                    let must_include = bitmask!(COLOR_ATTACHMENT_BIT | DEPTH_STENCIL_ATTACHMENT_BIT | INPUT_ATTACHMENT_BIT);
                    assert!(usage_flags.any_of(must_include), "VUID-VkImageCreateInfo-usage-00966");

                    let legal_transient_flags = bitmask!(TRANSIENT_ATTACHMENT_BIT | COLOR_ATTACHMENT_BIT | DEPTH_STENCIL_ATTACHMENT_BIT | INPUT_ATTACHMENT_BIT);
                    assert!(usage_flags.subset_of(legal_transient_flags), "VUID-VkImageCreateInfo-usage-00963")
                }

                if usage_flags.contains(SHADING_RATE_IMAGE_BIT_NV) {
                    assert!(!image_tiling.is(OPTIMAL), "VUID-VkImageCreateInfo-shadingRateImage-07727")
                }

                let sparse_create_flags = bitmask!(SPARSE_BINDING_BIT | SPARSE_RESIDENCY_BIT | SPARSE_ALIASED_BIT);
                if create_flags.any_of(sparse_create_flags) {
                    assert!(!usage_flags.contains(TRANSIENT_ATTACHMENT_BIT), "VUID-VkImageCreateInfo-None-01925")
                }
            };
            const VUID_vkGetPhysicalDeviceImageFormatProperties_usage_requiredbitmask: () = {
                let usage_flags = vk::raw_bitmask_from_type!(Usage);
                assert!(!usage_flags.is_empty())
            };
        }
    );

    check_vuid_defs!(
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_02248 : & 'static [ u8 ] = "tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2 instead)." . as_bytes ( ) ;
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_physicalDevice_parameter:
            &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_format_parameter: &'static [u8] =
            "format must be a valid VkFormat value".as_bytes();
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_type_parameter: &'static [u8] =
            "type must be a valid VkImageType value".as_bytes();
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_tiling_parameter: &'static [u8] =
            "tiling must be a valid VkImageTiling value".as_bytes();
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_usage_parameter: &'static [u8] =
            "usage must be a valid combination of VkImageUsageFlagBits values".as_bytes();
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_usage_requiredbitmask:
            &'static [u8] = "usage must not be 0".as_bytes();
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_flags_parameter: &'static [u8] =
            "flags must be a valid combination of VkImageCreateFlagBits values".as_bytes();
        pub const VUID_vkGetPhysicalDeviceImageFormatProperties_pImageFormatProperties_parameter:
            &'static [u8] =
            "pImageFormatProperties must be a valid pointer to a VkImageFormatProperties structure"
                .as_bytes();
    );
}

simple_struct_wrapper_scoped!(ImageFormatProperties impl Debug);