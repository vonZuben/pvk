use super::*;
use crate::instance::InstanceConfig;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;
use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html

VUID-vkGetPhysicalDeviceImageFormatProperties-tiling-02248
tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2 instead)

- checked with const Verify check

VUID-vkGetPhysicalDeviceImageFormatProperties-physicalDevice-parameter
physicalDevice must be a valid VkPhysicalDevice handle

- provided by vk_safe::PhysicalDevice

VUID-vkGetPhysicalDeviceImageFormatProperties-format-parameter
format must be a valid VkFormat value

- provided by vk::Format

VUID-vkGetPhysicalDeviceImageFormatProperties-type-parameter
type must be a valid VkImageType value

- provided by vk::ImageType

VUID-vkGetPhysicalDeviceImageFormatProperties-tiling-parameter
tiling must be a valid VkImageTiling value

- provided by vk::ImageTiling

VUID-vkGetPhysicalDeviceImageFormatProperties-usage-parameter
usage must be a valid combination of VkImageUsageFlagBits values

- const verify

VUID-vkGetPhysicalDeviceImageFormatProperties-usage-requiredbitmask
usage must not be 0

- const verify

VUID-vkGetPhysicalDeviceImageFormatProperties-flags-parameter
flags must be a valid combination of VkImageCreateFlagBits values

- TODO

VUID-vkGetPhysicalDeviceImageFormatProperties-pImageFormatProperties-parameter
pImageFormatProperties must be a valid pointer to a VkImageFormatProperties structure

- MaybeUninit
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C>
where
    C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceImageFormatProperties>,
{
    #[track_caller]
    pub fn get_physical_device_image_format_properties(
        &self,
        format: impl vk::FormatConst,
        image_type: impl vk::ImageTypeConst,
        image_tiling: impl vk::ImageTilingConst,
        usage_flags: impl vk::ImageUsageFlagsConst,
        create_flags: impl vk::ImageCreateFlagsConst,//vk::ImageCreateFlags,
    ) -> Result<ImageFormatProperties, vk::Result> {
        Params::verify(image_tiling, usage_flags, create_flags, image_type, format);
        let mut properties = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.feature_commands.get().get_fptr()(
                self.handle,
                format.variant(),
                image_type.variant(),
                image_tiling.variant(),
                usage_flags.bitmask(),
                create_flags.bitmask(),
                properties.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(ImageFormatProperties {
                inner: properties.assume_init(),
            })
        }
    }
}

verify_params!(Params(
    Tiling: vk::ImageTilingConst,
    Usage: vk::ImageUsageFlagsConst,
    Create: vk::ImageCreateFlagsConst,
    Type: vk::ImageTypeConst,
    Format: vk::FormatConst)
{
    use vk::image_usage_flag_bits::*;
    use vk::image_create_flag_bits::*;
    use vk::image_tiling::*;
    use vk::image_type::*;

    let image_tiling = vk::const_enum!(Tiling);
    let usage_flags = vk::raw_flags!(Usage);
    let create_flags = vk::raw_flags!(Create);
    let image_ty = vk::const_enum!(Type);

    // validate image_tiling
    if image_tiling.is(DRM_FORMAT_MODIFIER_EXT) {
        panic!("image_tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2 instead)");
    }

    // validate usage_flags
    {
        if usage_flags.is_empty() {
            panic!("usage_flags must not be 0");
        }

        if usage_flags.contains(TRANSIENT_ATTACHMENT_BIT) {
            let must_include = bitmask!(COLOR_ATTACHMENT_BIT | DEPTH_STENCIL_ATTACHMENT_BIT | INPUT_ATTACHMENT_BIT);
            if !usage_flags.any_of(must_include) {
                panic!("usage_flags must be a valid combination of VkImageUsageFlagBits values (see VUID-VkImageCreateInfo-usage-00966)")
            }

            let legal_transient_flags = bitmask!(TRANSIENT_ATTACHMENT_BIT | COLOR_ATTACHMENT_BIT | DEPTH_STENCIL_ATTACHMENT_BIT | INPUT_ATTACHMENT_BIT);
            if !usage_flags.subset_of(legal_transient_flags) {
                panic!("usage_flags must be a valid combination of VkImageUsageFlagBits values (see VUID-VkImageCreateInfo-usage-00963)")
            }
        }

        if usage_flags.contains(SHADING_RATE_IMAGE_BIT_NV) && !image_tiling.is(OPTIMAL) {
            panic!("if usage includes VK_IMAGE_USAGE_SHADING_RATE_IMAGE_BIT_NV, tiling must be VK_IMAGE_TILING_OPTIMAL (VUID-VkImageCreateInfo-shadingRateImage-07727)")
        }
    }

    // validate usage and create flags
    {
        let sparse_create_flags = bitmask!(SPARSE_BINDING_BIT | SPARSE_RESIDENCY_BIT | SPARSE_ALIASED_BIT);

        if create_flags.any_of(sparse_create_flags) && usage_flags.contains(TRANSIENT_ATTACHMENT_BIT) {
            panic!("usage_flags and create_flags must be a valid combination of values (see VUID-VkImageCreateInfo-None-01925)")
        }
    }

    /*
    VUID-VkImageCreateInfo-imageType-02082
    If usage includes VK_IMAGE_USAGE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR, imageType must be VK_IMAGE_TYPE_2D
    NOTE: currently not generating VK_IMAGE_USAGE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR

    VUID-VkImageCreateInfo-samples-02083
    If usage includes VK_IMAGE_USAGE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR, samples must be VK_SAMPLE_COUNT_1_BIT

    ^^^^^^^^^^^^^NOTE: currently not generating VK_IMAGE_USAGE_FRAGMENT_SHADING_RATE_ATTACHMENT_BIT_KHR^^^^^^^^^^^^^

    VUID-VkImageCreateInfo-usage-04992
    If usage includes VK_IMAGE_USAGE_INVOCATION_MASK_BIT_HUAWEI, tiling must be VK_IMAGE_TILING_LINEAR

    ^^^^^^^^^^^^^NOTE: not yet VK_IMAGE_USAGE_INVOCATION_MASK_BIT_HUAWEI^^^^^^^^^^^^^
     */

    if create_flags.contains(CUBE_COMPATIBLE_BIT) {
        assert!(image_ty.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-00949")
    }

    if usage_flags.contains(FRAGMENT_DENSITY_MAP_BIT_EXT) {
        assert!(image_ty.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02557")
    }

    if create_flags.contains(TYPE_2D_ARRAY_COMPATIBLE_BIT) {
        assert!(image_ty.is(TYPE_3D), "VUID-VkImageCreateInfo-flags-00950")
    }

    // TODO: VK_IMAGE_CREATE_2D_VIEW_COMPATIBLE_BIT_EXT -> VUID-VkImageCreateInfo-flags-07755

    if image_tiling.is(LINEAR) {
        assert!(!create_flags.contains(SPARSE_RESIDENCY_BIT), "VUID-VkImageCreateInfo-tiling-04121")
    }

    if image_ty.is(TYPE_1D) {
        assert!(!create_flags.contains(SPARSE_RESIDENCY_BIT), "VUID-VkImageCreateInfo-imageType-00970")
    }

    let sparse_bits = bitmask!(SPARSE_ALIASED_BIT | SPARSE_RESIDENCY_BIT);
    if create_flags.any_of(sparse_bits) {
        assert!(create_flags.contains(SPARSE_BINDING_BIT), "VUID-VkImageCreateInfo-flags-00987")
    }

    if create_flags.contains(SPLIT_INSTANCE_BIND_REGIONS_BIT) {
        assert!(image_ty.is(TYPE_2D), "VUID-VkImageCreateInfo-flags-02259") // also, mipLevels must be one, arrayLayers must be one, and imageCreateMaybeLinear (as defined in Image Creation Limits) must be VK_FALSE
    }

    if create_flags.contains(BLOCK_TEXEL_VIEW_COMPATIBLE_BIT) {
        assert!(Format::COMPRESSED_FORMAT, "VUID-VkImageCreateInfo-flags-01572");
        assert!(create_flags.contains(MUTABLE_FORMAT_BIT), "VUID-VkImageCreateInfo-flags-01573");
    }

    // VUID-VkImageCreateInfo-imageCreateFormatFeatures-02260 requires runtime check

    if !Format::MULTI_PLANAR && !create_flags.contains(ALIAS_BIT) {
        assert!(!create_flags.contains(DISJOINT_BIT), "VUID-VkImageCreateInfo-format-01577")
    }

    // VUID-VkImageCreateInfo-tiling-02353 need to deal with pnext

    if create_flags.contains(SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT) {
        assert!(Format::HAS_DEPTH_COMPONENT, "VUID-VkImageCreateInfo-flags-01533")
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

    // VUID-VkImageCreateInfo-imageView2DOn3DImage-04459 dynamic check

    // VUID-VkImageCreateInfo-pNext-06722 this is an interesting one to check, need pnext handling and more format properties
});

simple_struct_wrapper!(ImageFormatProperties);

impl fmt::Debug for ImageFormatProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
