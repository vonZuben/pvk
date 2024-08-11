use super::PhysicalDevice;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::structs::{
    ImageFormatProperties, ImageParameters::ImageParameters, SparseImageFormatProperties,
};

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceSparseImageFormatProperties;

unit_error!(OnlyOneSampleCountAllowed);
unit_error!(UnsupportedSampleCount);

pub(crate) fn get_physical_device_sparse_image_format_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceSparseImageFormatProperties>,
    A: ArrayStorage<SparseImageFormatProperties<P>>,
    Params: ImageParameters,
    SampleCount: vk::flag_traits::SampleCountFlags,
>(
    physical_device: &P,
    _samples: SampleCount,
    image_format_properties: ImageFormatProperties<P, Params>,
    mut storage: A,
) -> Result<A::InitStorage, Error> {
    check_vuids::check_vuids!(GetPhysicalDeviceSparseImageFormatProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_samples_01094: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "samples must be a bit value that is set in VkImageFormatProperties::sampleCounts returned"
        "by vkGetPhysicalDeviceImageFormatProperties with format, type, tiling, and usage equal"
        "to those in this command and flags equal to the value that is set in VkImageCreateInfo::flags"
        "when the image is created"
        }

        // I interpret this VUID to mean there should be exactly one bit set which is supported for the given image format, type, tiling, and usage
        if SampleCount::INCLUDES.count_bits() != 1 {
            Err(OnlyOneSampleCountAllowed)?
        } else if !image_format_properties
            .sample_counts
            .contains(SampleCount::INCLUDES)
        {
            Err(UnsupportedSampleCount)?
        }
        // since we keep the parameters used for the given ImageFormatProperties, we ensure to use the the same format, type, tiling, and usage
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_format_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "format must be a valid VkFormat value"
        }

        // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_type_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "type must be a valid VkImageType value"
        }

        // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_samples_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "samples must be a valid VkSampleCountFlagBits value"
        }

        // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_usage_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "usage must be a valid combination of VkImageUsageFlagBits values"
        }

        // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_usage_requiredbitmask: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "usage must not be 0"
        }

        // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_tiling_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "tiling must be a valid VkImageTiling value"
        }

        // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_pPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceSparseImageFormatProperties_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties"
        "must be a valid pointer to an array of pPropertyCount VkSparseImageFormatProperties"
        "structures"
        }

        // enumerator_code2!
    }

    enumerator_code2!(physical_device.commands().GetPhysicalDeviceSparseImageFormatProperties().get_fptr();
            (
                physical_device.raw_handle(),
                Params::format(),
                Params::image_type(),
                SampleCount::INCLUDES,
                Params::image_usage_flags(),
                Params::image_tiling()
            )
            -> storage)
}
