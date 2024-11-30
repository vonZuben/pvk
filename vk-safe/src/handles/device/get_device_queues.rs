use super::Device;

use std::convert::TryInto;
use std::mem::MaybeUninit;

use crate::scope::{Captures, Tag};
use crate::structs::QueueFamiliesRef;
use crate::vk::DeviceQueueCreateInfo;
use crate::vk::{make_queue, Queue, QueueFamilyMarker};

use vk_safe_sys as vk;

use vk::flag_traits::QueueFlags;
use vk::has_command::GetDeviceQueue;

unit_error!(
/// QueueFamily does not support the desired capabilities
pub UnsupportedCapability
);

/// Get the Queues configured for a device in a queue family
///
/// Must provide the [`DeviceQueueCreateInfo`] that was used when creating the device.
/// The Queues for each family can only be obtained once, and so the DeviceQueueCreateInfo
/// is consumed and cannot be used again.
///
/// The returned Queues
pub fn get_device_queues<'a, 't, D: Device<Commands: GetDeviceQueue>, Q: QueueFlags>(
    device: &'a D,
    family_config: DeviceQueueCreateInfo<D::QueueConfig>,
    queue_family_properties: &QueueFamiliesRef<D::PhysicalDevice>,
    capability: Q,
    tag: Tag<'t>,
) -> Result<
    (
        QueueFamilyMarker<Tag<'t>>,
        impl Iterator<Item: Queue<Device = D, Capability = Q, Family = Tag<'t>> + Captures<&'a D>>,
    ),
    UnsupportedCapability,
> {
    let family_index: u32 = family_config.queue_family_index;
    let family_flags = unsafe {
        // The family index is valid because the Device
        // was created with the same que config, which
        // comes from the same physical device
        queue_family_properties
            .get_unchecked(family_index as usize)
            .queue_flags
    };

    if family_flags.satisfies(capability) {
        let num_queues: u32 = family_config
            .queue_priorities()
            .len()
            .try_into()
            .expect("this should already be valid u32 from creating DeviceQueueCreateInfo");
        let mut i = 0;
        let fptr = device.commands().GetDeviceQueue().get_fptr();
        let device_handle = device.raw_handle();

        let queue_family_marker = unsafe { QueueFamilyMarker::new(family_index, &tag) };
        let queue_iter = std::iter::from_fn(move || {
            if i == num_queues {
                None
            } else {
                let mut handle = MaybeUninit::uninit();
                unsafe {
                    fptr(device_handle, family_index, i, handle.as_mut_ptr());
                    i += 1;
                    Some(make_queue(handle.assume_init(), device, &tag))
                }
            }
        });

        Ok((queue_family_marker, queue_iter))
    } else {
        Err(UnsupportedCapability)
    }
}
