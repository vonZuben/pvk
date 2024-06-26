use super::*;

use concrete_type::DeviceConfig;

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use vk::has_command::GetDeviceQueue;
use vk_safe_sys as vk;

use crate::dispatchable_handles::queue::{
    concrete_type::Config, concrete_type::Queue, QueueCapability,
};
use crate::vk::DeviceQueueCreateInfo;

impl<'a, S, C: concrete_type::DeviceConfig> concrete_type::ScopedDevice<S, C> {
    /// get the configured queue families
    ///
    /// In Vulkan, after creating a device, you normally use `vkGetDeviceQueue`
    /// to get the queues that you configured during device creation.
    ///
    /// vk-safe does not have an exact method corresponding to `vkGetDeviceQueue`
    ///
    /// use this method instead to get the queue families that you configured.
    /// Each queue family is represented with a [`QueueFamily`] which allows you
    /// to get your configured queues after you verify the [`QueueFlags`](crate::vk::QueueFlags).
    pub fn get_configured_queue_families(
        &self,
    ) -> impl Iterator<Item = QueueFamily<S, Unknown>> + crate::scope::Captures<&Self> {
        self.config
            .queue_config()
            .iter()
            .enumerate()
            .map(move |(i, _)| QueueFamily {
                config_index: i,
                device: self.scope_ref(),
                capability: PhantomData,
            })
    }
}

unit_error!(pub QueueIndexNotConfigured);

impl<D: Device, Q: QueueCapability> QueueFamily<'_, D, Q>
where
    D::Context: GetDeviceQueue,
{
    /// Get a queue from a QueueFamily with known [`QueueFlags`](crate::vk::QueueFlags)
    ///
    /// After determining what operations a QueueFamily supports, call this
    /// method to get individual queues. This is where `vkGetDeviceQueue` will
    /// actually be called.
    ///
    /// see also <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetDeviceQueue.html>
    pub fn get_queue(
        &self,
        queue_index: u32,
    ) -> Result<Queue<Config<D, Q>>, QueueIndexNotConfigured> {
        let config = self.queue_config();
        if queue_index < config.inner.queue_count {
            let family_index = config.inner.queue_family_index;
            let mut queue = MaybeUninit::uninit();
            unsafe {
                let fptr = self.device.context.GetDeviceQueue().get_fptr();
                fptr(
                    self.device.handle,
                    family_index,
                    queue_index,
                    queue.as_mut_ptr(),
                );
                Ok(Queue::new(queue.assume_init(), Config::new(self.device)))
            }
        } else {
            Err(QueueIndexNotConfigured)
        }
    }
}

#[derive(Debug)]
pub struct Unknown;

/// A configured queue family
///
/// provides access to individual queues in the family
#[derive(Clone, Copy)]
pub struct QueueFamily<'a, D, Q> {
    config_index: usize,
    device: &'a D,
    capability: PhantomData<Q>,
}

impl<D: Device, Q> std::fmt::Debug for QueueFamily<'_, D, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueFamily")
            .field("config", self.queue_config())
            .field("queue_family_properties", self.queue_family_properties())
            .finish()
    }
}

unit_error!(pub CapabilityNotSupported);

impl<'a, D: Device, U> QueueFamily<'a, D, U> {
    fn queue_config(&self) -> &DeviceQueueCreateInfo<D::PhysicalDevice> {
        unsafe {
            self.device
                .config
                .queue_config()
                .get_unchecked(self.config_index)
        }
    }

    fn queue_family_properties(&self) -> &vk::QueueFamilyProperties {
        unsafe {
            self.device
                .config
                .queue_family_properties()
                .get_unchecked(self.queue_config().inner.queue_family_index as usize)
            // 'as' cast should be fine at this point since we should already know the index is valid
        }
    }

    /// Ensure the QueueFamily has supports specific operations
    ///
    /// [`QueueFlags`](crate::vk::QueueFlags) represents the operations that queues in the family supports.
    /// Call this method to verify in the type system that this QueueFamily supports the
    /// operations you want to use.
    ///
    /// see <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkQueueFlagBits.html>
    pub fn with_capability<Q: QueueCapability>(
        self,
        _capability: Q,
    ) -> Result<QueueFamily<'a, D, Q>, CapabilityNotSupported> {
        if self
            .queue_family_properties()
            .queue_flags
            .contains(Q::INCLUDES)
        {
            Ok(QueueFamily {
                config_index: self.config_index,
                device: self.device,
                capability: PhantomData,
            })
        } else {
            Err(CapabilityNotSupported)
        }
    }
}
