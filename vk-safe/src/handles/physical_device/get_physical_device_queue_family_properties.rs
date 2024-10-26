use super::PhysicalDevice;

use crate::buffer::Buffer;
use crate::enumerator::{Enumerator, EnumeratorTarget};
use crate::error::Error;
use crate::scope::Captures;
use crate::structs::QueueFamilies;

use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceQueueFamilyProperties;

pub(crate) fn get_physical_device_queue_family_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceQueueFamilyProperties>,
>(
    physical_device: &P,
) -> impl Enumerator<vk::QueueFamilyProperties, QueueFamiliesTarget<P>> + Captures<&P> {
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

    struct QueueFamilyPropertiesEnumerator<F>(F);

    impl<F, P> Enumerator<vk::QueueFamilyProperties, QueueFamiliesTarget<P>>
        for QueueFamilyPropertiesEnumerator<F>
    where
        F: Fn(&mut u32, *mut vk::QueueFamilyProperties),
    {
        fn get_len(&self) -> Result<usize, Error> {
            let mut len = 0;
            // UNSAFE warning
            // the call to this is actually unsafe, but can't be reflected with the Fn trait
            // However, this can only be used internal to this macro, so it is fine
            let res = self.0(&mut len, std::ptr::null_mut());
            check_raw_err!(res);
            Ok(len.try_into()?)
        }

        fn get_enumerate<B: Buffer<vk::QueueFamilyProperties>>(
            &self,
            mut buffer: B,
        ) -> Result<QueueFamilies<P, B>, Error> {
            let mut len = buffer.capacity().try_into()?;
            // UNSAFE warning
            // the call to this is actually unsafe, but can't be reflected with the Fn trait
            // However, this can only be used internal to this macro, so it is fine
            let res = self.0(&mut len, buffer.ptr_mut());
            check_raw_err!(res);
            unsafe {
                buffer.set_len(len.try_into()?);
            }
            Ok(QueueFamilies::new(buffer))
        }
    }

    QueueFamilyPropertiesEnumerator(move |len: &mut _, buffer: *mut _| unsafe {
        physical_device
            .commands()
            .GetPhysicalDeviceQueueFamilyProperties()
            .get_fptr()(physical_device.raw_handle(), len, buffer)
    })
}

/// impl Enumerator target for GetPhysicalDeviceQueueFamilyProperties
pub struct QueueFamiliesTarget<S>(PhantomData<S>);

impl<S> EnumeratorTarget for QueueFamiliesTarget<S> {
    type Target<B> = QueueFamilies<S, B>;
}
