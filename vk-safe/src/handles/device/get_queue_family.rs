use super::Device;

use std::convert::TryInto;
use std::marker::PhantomData;

use crate::scope::Tag;
use crate::structs::QueueFamiliesRef;
use crate::vk::DeviceQueueCreateInfo;
use crate::vk::{_Queue, make_queue, QueueCapability};

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
) -> Result<_QueueFamily<'a, D, Q, Tag<'t>>, UnsupportedCapability> {
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
        Ok(_QueueFamily {
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
///
/// You are not intended to implement this trait. The trait
/// is only implemented by an internal only type.
pub trait QueueFamily<'device>: std::fmt::Debug + Send + Sync {
    type Capability;
    type Tag;
    type Device;

    /// Get the number of queues in this queue family
    fn num_queues(&self) -> u32;

    /// get the family index of this queue
    fn family_index(&self) -> u32;

    /// Get an individual queue with the provided index
    ///
    /// Will return a [`Queue`](crate::vk::Queue) if the index <= number of
    /// created queues. Will return [`InvalidIndex`] otherwise.
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetDeviceQueue.html>
    fn get_device_queue(
        &self,
        index: u32,
    ) -> Result<
        // impl Queue<Commands = D::Commands, Device = D, Capability = C> + Captures<&'a D>,
        _Queue<'device, Self::Device, Self::Capability, Self::Tag>,
        InvalidIndex,
    >;
}

/// [`QueueFamily`] implementor
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT. After some kind of precise capturing is possible,
/// this type will be made private and <code>impl [QueueFamily]</code>
/// will be returned.
#[derive(Clone, Copy)]
pub struct _QueueFamily<'a, D, C, T> {
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

impl<'a, D: Sync, C: Sync + Send, T: Sync + Send> QueueFamily<'a> for _QueueFamily<'a, D, C, T>
where
    D: Device<Commands: GetDeviceQueue>,
    C: QueueCapability,
{
    type Capability = C;
    type Tag = T;
    type Device = D;

    fn family_index(&self) -> u32 {
        self.family_index
    }

    fn num_queues(&self) -> u32 {
        todo!()
    }

    fn get_device_queue(
        &self,
        index: u32,
    ) -> Result<_Queue<'a, Self::Device, Self::Capability, Self::Tag>, InvalidIndex> {
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

impl<D: Device, Q: QueueCapability, T> std::fmt::Debug for _QueueFamily<'_, D, Q, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueFamily")
            .field("number of queues", &self.num_queues)
            .field("family index", &self.family_index)
            .field("capability", &Q::INCLUDES)
            .finish()
    }
}
