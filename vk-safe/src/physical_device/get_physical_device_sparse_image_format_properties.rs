use super::*;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceSparseImageFormatProperties;

use super::get_physical_device_image_format_properties::ImageFormatProperties;

use crate::array_storage::ArrayStorage;
use crate::error::Error;

impl<'scope, I: Instance> ScopedPhysicalDeviceType<'scope, I> {
    pub fn get_physical_device_sparse_image_format_properties<
        P,
        S: ArrayStorage<SparseImageFormatProperties<'scope>>,
    >(
        &self,
        samples: vk::SampleCountFlags,
        image_format_properties: ImageFormatProperties<'scope>,
        mut storage: S,
    ) -> Result<S::InitStorage, Error>
    where
        I::Commands: GetPhysicalDeviceSparseImageFormatProperties<P>,
    {
        check_vuids::check_vuids!(GetPhysicalDeviceSparseImageFormatProperties);
        // check_vuid_defs2!(GetPhysicalDeviceSparseImageFormatProperties
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_samples_01094 : & 'static [ u8 ] =
        //         "samples must be a bit value that is set in VkImageFormatProperties::sampleCounts returned by vkGetPhysicalDeviceImageFormatProperties with format, type, tiling, and usage equal to those in this command and flags equal to the value that is set in VkImageCreateInfo::flags when the image is created" . as_bytes ( ) ;
        //         CHECK {
        //             // I interpret this VUID to mean there should be exactly one bit set which is supported for the given image format, type, tiling, and usage
        //             assert!(samples.count_bits() == 1);
        //             assert!(image_format_properties.inner.sample_counts.contains(samples));
        //             // since we keep the parameters used for the given ImageFormatProperties, we ensure to use the the same format, type, tiling, and usage
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_physicalDevice_parameter:
        //         &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        //         CHECK {
        //             // ensure by PhysicalDevice creation
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_format_parameter:
        //         &'static [u8] = "format must be a valid VkFormat value".as_bytes();
        //         CHECK {
        //             // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_type_parameter:
        //         &'static [u8] = "type must be a valid VkImageType value".as_bytes();
        //         CHECK {
        //             // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_samples_parameter:
        //         &'static [u8] = "samples must be a valid VkSampleCountFlagBits value".as_bytes();
        //         CHECK {
        //             // ensured by vk::SampleCountFlags
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_usage_parameter:
        //         &'static [u8] =
        //         "usage must be a valid combination of VkImageUsageFlagBits values".as_bytes();
        //         CHECK {
        //             // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_usage_requiredbitmask:
        //         &'static [u8] = "usage must not be 0".as_bytes();
        //         CHECK {
        //             // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_tiling_parameter:
        //         &'static [u8] = "tiling must be a valid VkImageTiling value".as_bytes();
        //         CHECK {
        //             // checked in ImageFormatProperties.GetPhysicalDeviceImageFormatPropertiesParameters
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_pPropertyCount_parameter:
        //         &'static [u8] = "pPropertyCount must be a valid pointer to a uint32_t value".as_bytes();
        //         CHECK {
        //             // handled by enumerator_code2!()
        //         }
        //     pub const VUID_vkGetPhysicalDeviceSparseImageFormatProperties_pProperties_parameter : & 'static [ u8 ] =
        //         "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkSparseImageFormatProperties structures" . as_bytes ( ) ;
        //         CHECK {
        //             // handled by enumerator_code2!()
        //         }
        // );

        enumerator_code2!(self.instance.commands.GetPhysicalDeviceSparseImageFormatProperties().get_fptr();
            (
                self.handle,
                image_format_properties.params.format,
                image_format_properties.params.image_type,
                samples,
                image_format_properties.params.usage_flags,
                image_format_properties.params.image_tiling
            )
            -> storage)
    }
}

simple_struct_wrapper_scoped!(SparseImageFormatProperties impl Debug, Deref);
