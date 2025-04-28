use crate::enumerator::Enumerator;
use crate::error::Error;
use crate::handles::DispatchableHandle;
use crate::scope::Captures;
use crate::structs::{
    ImageFormatProperties, ImageParameters::ImageParameters, SparseImageFormatProperties,
};

use vk_safe_sys as vk;

unit_error!(OnlyOneSampleCountAllowed);
unit_error!(UnsupportedSampleCount);

pub trait GetPhysicalDeviceSparseImageFormatProperties:
    DispatchableHandle<RawHandle = vk::PhysicalDevice>
{
    /// Query the sparse image format properties of the PhysicalDevice
    ///
    /// ### Note
    /// This requires [`ImageFormatProperties`] from
    /// [`get_physical_device_image_format_properties`](PhysicalDevice::get_physical_device_image_format_properties()),
    /// which provides [`ImageParameters`] and ensures that the sample count you choose is supported
    /// by an image with such parameters.
    ///
    /// Must provide the storage space to return the properties to.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<
    /// #   P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>,
    /// #   Params: vk::ImageParameters::ImageParameters,
    /// # >
    /// #   (physical_device: P, image_format_properties: vk::ImageFormatProperties<P, Params>) {
    /// let sparse_image_format_properties =
    ///     physical_device.get_physical_device_sparse_image_format_properties(
    ///         vk::SampleCountFlags::TYPE_1_BIT,
    ///         image_format_properties,
    ///     )
    ///     .unwrap()
    ///     .auto_get_enumerate()
    ///     .unwrap();
    /// # }
    /// ```
    fn get_physical_device_sparse_image_format_properties<
        Params: ImageParameters,
        SampleCount: vk::flag_traits::SampleCountFlags,
        X,
    >(
        &self,
        _samples: SampleCount,
        image_format_properties: ImageFormatProperties<Self, Params>,
    ) -> Result<impl Enumerator<SparseImageFormatProperties<Self>> + Captures<&Self>, Error>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceSparseImageFormatProperties<X>,
    {
        use vk::has_command::GetPhysicalDeviceSparseImageFormatProperties;

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

        Ok(make_enumerator!(
            self.commands().GetPhysicalDeviceSparseImageFormatProperties().get_fptr();
                    (
                        self.raw_handle(),
                        Params::format(),
                        Params::image_type(),
                        SampleCount::INCLUDES,
                        Params::image_usage_flags(),
                        Params::image_tiling()
                    )
        ))
    }
}
