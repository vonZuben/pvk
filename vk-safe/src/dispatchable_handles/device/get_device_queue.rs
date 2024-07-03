/*!
Get Device Queues

Unlike the Vulkan C API, you do not obtain Queues directly from the Device. Rather, you first
call [`get_queue_family`](ScopedDevice::get_queue_family) to obtain [`QueueFamily`],
which represents a Queue Family that you configured when creating the Device. You can
then call [`get_device_queue`](QueueFamily::get_device_queue) from the `QueueFamily`
in order to get an <code>impl [Queue]</code>.

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetDeviceQueue.html>
*/

use super::*;

use concrete_type::{DeviceConfig, ScopedDevice};

use std::convert::TryInto;
use std::marker::PhantomData;

use vk::has_command::GetDeviceQueue;
use vk_safe_sys as vk;

use crate::dispatchable_handles::physical_device::get_physical_device_queue_family_properties::QueueFamiliesRef;
use crate::dispatchable_handles::queue;
use crate::vk::DeviceQueueCreateInfo;

use crate::scope::Captures;

pub use crate::dispatchable_handles::queue::{Queue, QueueCapability};

unit_error!(
/// QueueFamily does not support the desired capabilities
pub UnsupportedCapability
);

impl<'a, S, C: DeviceConfig> ScopedDevice<S, C> {
    /// Get a QueueFamily which should have specific capabilities
    ///
    /// In vk-safe, you do not directly get queues from the Device. Rather,
    /// you first get a type that represents a [`QueueFamily`] that you already
    /// configured by by passing in the same queue configuration and properties
    /// parameters used when creating the Device.
    ///
    /// From the returned [`QueueFamily`], you can obtain individual [`Queue`]
    /// objects.
    ///
    /// Returns the [`QueueFamily`] if the [`QueueCapability`] is supported.
    /// Otherwise returns [`UnsupportedCapability`].
    pub fn get_queue_family<Q: QueueCapability>(
        &self,
        queue_config: &DeviceQueueCreateInfo<C::QueueConfig>,
        queue_family_properties: &QueueFamiliesRef<C::PhysicalDevice>,
        capability: Q,
    ) -> Result<QueueFamily<S, Q>, UnsupportedCapability> {
        let _ = capability;
        let family: u32 = queue_config.queue_family_index;
        let family_flags = unsafe {
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
                device: self.scope_ref(),
                capability: PhantomData,
            })
        } else {
            Err(UnsupportedCapability)
        }
    }
}

/// A configured queue family
///
/// provides access to individual queues in the family
#[derive(Clone, Copy)]
pub struct QueueFamily<'a, S, Q> {
    num_queues: u32,
    family_index: u32,
    device: &'a S,
    capability: PhantomData<Q>,
}

unit_error!(
/// Queue index is >= number of available queues
pub InvalidIndex
);

impl<'a, S, Q> QueueFamily<'a, S, Q> {
    /// Get an individual queue with the provided index
    ///
    /// Will return a [`Queue`] if the index <= number of
    /// created queues. Will return [`InvalidIndex`] otherwise.
    pub fn get_device_queue(
        &self,
        index: u32,
    ) -> Result<impl Queue<Context = S::Context, Capability = Q> + Captures<&'a S>, InvalidIndex>
    where
        // NOTE: the type bounds are represented as a where clause on the method rather than
        // bounds on the impl block because I have found it works better with rust-analyzer autocomplete.
        // I do not believe there are any other practical effects (positive or negative) from this choice
        // in consideration of how this should normally be used.
        S: Device<Context: GetDeviceQueue>,
        Q: QueueCapability,
    {
        let mut queue = std::mem::MaybeUninit::uninit();
        let fptr = self.device.context.GetDeviceQueue().get_fptr();
        if index >= self.num_queues {
            return Err(InvalidIndex);
        }

        unsafe {
            fptr(
                self.device.handle,
                self.family_index,
                index,
                queue.as_mut_ptr(),
            );
            Ok(queue::concrete_type::Queue::new(
                queue.assume_init(),
                queue::concrete_type::Config::<S, Q>::new(self.device),
            ))
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
