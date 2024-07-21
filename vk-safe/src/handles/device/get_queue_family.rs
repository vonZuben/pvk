use super::Device;

use std::convert::TryInto;
use std::marker::PhantomData;

use crate::scope::{Captures, Tag};
use crate::structs::QueueFamiliesRef;
use crate::vk::DeviceQueueCreateInfo;
use crate::vk::{make_queue, Queue, QueueCapability};

use vk_safe_sys as vk;

use vk::has_command::GetDeviceQueue;

unit_error!(
/// QueueFamily does not support the desired capabilities
pub UnsupportedCapability
);

pub(crate) fn get_queue_family<'a, 't, D: Device, Q: QueueCapability>(
    device: &'a D,
    queue_config: &DeviceQueueCreateInfo<D::QueueConfig>,
    queue_family_properties: &QueueFamiliesRef<D::PhysicalDevice>,
    _capability: Q,
    _tag: Tag<'t>,
) -> Result<QueueFamily<'a, D, Q, Tag<'t>>, UnsupportedCapability> {
    let family: u32 = queue_config.queue_family_index;
    let family_flags = unsafe {
        // The family index is valid because the Device
        // was created with the same que config, which
        // comes from the same physical device
        queue_family_properties
            .get_unchecked(family as usize)
            .queue_flags
    };

    if Q::satisfies(family_flags) {
        // this should already be valid from creating DeviceQueueCreateInfo
        let num_queues: u32 = queue_config.queue_priorities().len().try_into().unwrap();
        Ok(QueueFamily {
            num_queues,
            family_index: queue_config.queue_family_index,
            device,
            capability: PhantomData,
            tag: PhantomData,
        })
    } else {
        Err(UnsupportedCapability)
    }
}

/// A configured queue family
///
/// provides access to individual queues in the family
#[derive(Clone, Copy)]
pub struct QueueFamily<'a, D, C, T> {
    num_queues: u32,
    family_index: u32,
    device: &'a D,
    capability: PhantomData<C>,
    tag: PhantomData<T>,
}

unit_error!(
/// Queue index is >= number of available queues
pub InvalidIndex
);

impl<'a, D, C, T> QueueFamily<'a, D, C, T> {
    /// Get an individual queue with the provided index
    ///
    /// Will return a [`Queue`] if the index <= number of
    /// created queues. Will return [`InvalidIndex`] otherwise.
    pub fn get_device_queue(
        &self,
        index: u32,
    ) -> Result<
        impl Queue<Commands = D::Commands, Device = D, Capability = C> + Captures<&'a D>,
        InvalidIndex,
    >
    where
        // NOTE: the type bounds are represented as a where clause on the method rather than
        // bounds on the impl block because I have found it works better with rust-analyzer autocomplete.
        // I do not believe there are any other practical effects (positive or negative) from this choice
        // in consideration of how this should normally be used.
        D: Device<Commands: GetDeviceQueue>,
        C: QueueCapability,
    {
        check_vuids::check_vuids!(GetDeviceQueue);

        #[allow(unused_labels)]
        'VUID_vkGetDeviceQueue_queueFamilyIndex_00384: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueFamilyIndex must be one of the queue family indices specified when device was"
            "created, via the VkDeviceQueueCreateInfo structure"
            }

            // ensured by get_queue_family which uses the same DeviceQueueCreateInfo the device was created with
        }

        #[allow(unused_labels)]
        'VUID_vkGetDeviceQueue_queueIndex_00385: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueIndex must be less than the value of VkDeviceQueueCreateInfo::queueCount for"
            "the queue family indicated by queueFamilyIndex when device was created"
            }

            if index >= self.num_queues {
                return Err(InvalidIndex);
            }
        }

        #[allow(unused_labels)]
        'VUID_vkGetDeviceQueue_flags_01841: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "VkDeviceQueueCreateInfo::flags must have been set to zero when device was created"
            }

            // ensured by DeviceQueueCreateInfo creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetDeviceQueue_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "device must be a valid VkDevice handle"
            }

            // ensured by device creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetDeviceQueue_pQueue_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pQueue must be a valid pointer to a VkQueue handle"
            }

            // MaybeUninit
        }

        let mut queue = std::mem::MaybeUninit::uninit();
        let fptr = self.device.commands().GetDeviceQueue().get_fptr();
        unsafe {
            fptr(
                self.device.raw_handle(),
                self.family_index,
                index,
                queue.as_mut_ptr(),
            );
            Ok(make_queue(queue.assume_init(), self.device, self.tag))
        }
    }
}

impl<D: Device, Q: QueueCapability, T> std::fmt::Debug for QueueFamily<'_, D, Q, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueFamily")
            .field("number of queues", &self.num_queues)
            .field("family index", &self.family_index)
            .field("capability", &Q::INCLUDES)
            .finish()
    }
}
