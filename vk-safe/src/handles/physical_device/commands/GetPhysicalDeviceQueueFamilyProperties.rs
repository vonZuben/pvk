use crate::enumerator::{Enumerator, EnumeratorTarget};
use crate::handles::DispatchableHandle;
use crate::scope::Captures;
use crate::structs::QueueFamilies;

use std::marker::PhantomData;

use vk_safe_sys as vk;

pub trait GetPhysicalDeviceQueueFamilyProperties:
    DispatchableHandle<RawHandle = vk::PhysicalDevice>
{
    /// Query the queue family properties of the PhysicalDevice
    ///
    /// Must provide the storage space to return the properties to.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let queue_family_properties =
    ///     physical_device.get_physical_device_queue_family_properties()
    ///     .auto_get_enumerate()
    ///     .unwrap();
    /// # }
    /// ```
    fn get_physical_device_queue_family_properties<X>(
        &self,
    ) -> impl Enumerator<vk::QueueFamilyProperties, QueueFamiliesTarget<Self>> + Captures<&Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceQueueFamilyProperties<X>,
    {
        use vk::has_command::GetPhysicalDeviceQueueFamilyProperties;

        check_vuids::check_vuids!(GetPhysicalDeviceQueueFamilyProperties);

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceQueueFamilyProperties_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pQueueFamilyPropertyCount must be a valid pointer to a uint32_t value"
            }

            // enumerator_code2!
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the value referenced by pQueueFamilyPropertyCount is not 0, and pQueueFamilyProperties"
            "is not NULL, pQueueFamilyProperties must be a valid pointer to an array of pQueueFamilyPropertyCount"
            "VkQueueFamilyProperties structures"
            }

            // enumerator_code2!
        }

        let fptr = self
            .commands()
            .GetPhysicalDeviceQueueFamilyProperties()
            .get_fptr();

        make_enumerator!(fptr; (self.raw_handle()) )
    }
}

/// impl Enumerator target for GetPhysicalDeviceQueueFamilyProperties
pub struct QueueFamiliesTarget<S>(PhantomData<S>);

impl<S> EnumeratorTarget for QueueFamiliesTarget<S> {
    type Target<B> = QueueFamilies<S, B>;

    fn make_target<B>(buffer: B) -> Self::Target<B> {
        QueueFamilies::new(buffer)
    }
}
