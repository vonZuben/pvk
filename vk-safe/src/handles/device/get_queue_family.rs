use super::Device;

use std::convert::TryInto;
use std::marker::PhantomData;

use crate::scope::Captures;
use crate::structs::QueueFamiliesRef;
use crate::vk::DeviceQueueCreateInfo;
use crate::vk::{make_queue, Queue, QueueCapability};

use vk_safe_sys as vk;

use vk::has_command::GetDeviceQueue;

unit_error!(
/// QueueFamily does not support the desired capabilities
pub UnsupportedCapability
);

pub(crate) fn get_queue_family<'a, D: Device, Q: QueueCapability>(
    device: &'a D,
    queue_config: &DeviceQueueCreateInfo<D::QueueConfig>,
    queue_family_properties: &QueueFamiliesRef<D::PhysicalDevice>,
    _capability: Q,
) -> Result<QueueFamily<'a, D, Q>, UnsupportedCapability> {
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
        })
    } else {
        Err(UnsupportedCapability)
    }
}

/// A configured queue family
///
/// provides access to individual queues in the family
#[derive(Clone, Copy)]
pub struct QueueFamily<'a, D, C> {
    num_queues: u32,
    family_index: u32,
    device: &'a D,
    capability: PhantomData<C>,
}

unit_error!(
/// Queue index is >= number of available queues
pub InvalidIndex
);

impl<'a, D, C> QueueFamily<'a, D, C> {
    /// Get an individual queue with the provided index
    ///
    /// Will return a [`Queue`] if the index <= number of
    /// created queues. Will return [`InvalidIndex`] otherwise.
    pub fn get_device_queue(
        &self,
        index: u32,
    ) -> Result<impl Queue<Commands = D::Commands, Capability = C> + Captures<&'a D>, InvalidIndex>
    where
        // NOTE: the type bounds are represented as a where clause on the method rather than
        // bounds on the impl block because I have found it works better with rust-analyzer autocomplete.
        // I do not believe there are any other practical effects (positive or negative) from this choice
        // in consideration of how this should normally be used.
        D: Device<Commands: GetDeviceQueue>,
        C: QueueCapability,
    {
        let mut queue = std::mem::MaybeUninit::uninit();
        let fptr = self.device.commands().GetDeviceQueue().get_fptr();
        if index >= self.num_queues {
            return Err(InvalidIndex);
        }

        unsafe {
            fptr(
                self.device.raw_handle(),
                self.family_index,
                index,
                queue.as_mut_ptr(),
            );
            Ok(make_queue(queue.assume_init(), self.device))
        }
    }
}

impl<D: Device, Q: QueueCapability> std::fmt::Debug for QueueFamily<'_, D, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueFamily")
            .field("number of queues", &self.num_queues)
            .field("family index", &self.family_index)
            .field("capability", &Q::INCLUDES)
            .finish()
    }
}
