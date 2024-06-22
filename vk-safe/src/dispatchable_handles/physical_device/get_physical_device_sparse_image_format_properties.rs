/*!
Query the sparse image format properties of the PhysicalDevice

use the [`get_physical_device_sparse_image_format_properties`](ScopedPhysicalDevice::get_physical_device_sparse_image_format_properties) method on a scoped PhysicalDevice

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceSparseImageFormatProperties.html>
*/

use super::concrete_type::ScopedPhysicalDevice;
use super::PhysicalDeviceConfig;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceSparseImageFormatProperties;

use super::get_physical_device_image_format_properties::ImageFormatProperties;

use crate::array_storage::ArrayStorage;
use crate::error::Error;

impl<S, C: PhysicalDeviceConfig> ScopedPhysicalDevice<S, C>
where
    C::Context: GetPhysicalDeviceSparseImageFormatProperties,
{
    /**
    Query the sparse image format properties of the PhysicalDevice

    ### Note
    *this currently takes [`ImageFormatProperties`] which needs to be obtained in advance from
    [`get_physical_device_image_format_properties`](ScopedPhysicalDevice::get_physical_device_image_format_properties).
    However this should probably be changed. I am considering a general purpose "image parameters" type which may replace
    [`GetPhysicalDeviceImageFormatPropertiesParameters`](crate::vk::GetPhysicalDeviceImageFormatPropertiesParameters) in future.*

    Must provide the storage space to return the properties to.

    ```rust
    # use vk_safe::vk;
    # vk::device_context!(D: VERSION_1_0);
    # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    #   (physical_device: P, image_format_properties: vk::ImageFormatProperties<P>) {
    let sparse_image_format_properties =
        physical_device.get_physical_device_sparse_image_format_properties(
            vk::SampleCountFlags::TYPE_1_BIT,
            image_format_properties,
            Vec::new(),
        );
    # }
    ```
    */
    pub fn get_physical_device_sparse_image_format_properties<
        A: ArrayStorage<SparseImageFormatProperties<S>>,
    >(
        &self,
        samples: vk::SampleCountFlags,
        image_format_properties: ImageFormatProperties<S>,
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
            assert!(samples.count_bits() == 1);
            assert!(image_format_properties
                .inner
                .sample_counts
                .contains(samples));
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

        enumerator_code2!(self.instance().context.GetPhysicalDeviceSparseImageFormatProperties().get_fptr();
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
