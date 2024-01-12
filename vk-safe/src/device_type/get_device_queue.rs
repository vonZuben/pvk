use super::*;

use std::mem::MaybeUninit;

use vk::has_command::GetDeviceQueue;
use vk::queue_capability::QueueCapability;

use crate::queue_type::{Config, QueueType};

use crate::DeviceQueueCreateInfo;

impl<'a, S, C: DeviceConfig> ScopedDeviceType<S, C> {
    /// get the configured queue families
    pub fn get_configured_queue_families(
        &self,
    ) -> impl Iterator<Item = QueueFamily<S, Unknown>> + crate::scope::Captures<&Self> {
        self.config
            .queue_config()
            .iter()
            .enumerate()
            .map(|(i, _)| QueueFamily {
                config_index: i,
                device: self.as_scope(),
                capability: PhantomData,
            })
    }
}

unit_error!(pub QueueIndexOutOfRange);

impl<D: Device, Q: QueueCapability> QueueFamily<D, Q>
where
    D::Commands: GetDeviceQueue,
{
    pub fn get_queue_family(
        &self,
        queue_index: u32,
    ) -> Result<QueueType<Config<D, Q>>, QueueIndexOutOfRange> {
        if queue_index < self.queue_family_properties().queue_count {
            let family_index = self.queue_config().inner.queue_family_index;
            let mut queue = MaybeUninit::uninit();
            unsafe {
                let fptr = self.device.commands.GetDeviceQueue().get_fptr();
                fptr(
                    self.device.handle,
                    family_index,
                    queue_index,
                    queue.as_mut_ptr(),
                );
                Ok(QueueType::new(queue.assume_init(), self.device))
            }
        } else {
            Err(QueueIndexOutOfRange)
        }
    }
}

#[derive(Debug)]
pub struct Unknown;

#[derive(Clone, Copy)]
pub struct QueueFamily<D, Q> {
    config_index: usize,
    device: D,
    capability: PhantomData<Q>,
}

impl<D: Device, Q> std::fmt::Debug for QueueFamily<D, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueFamily")
            .field("config", self.queue_config())
            .field("queue_family_properties", self.queue_family_properties())
            .finish()
    }
}

unit_error!(pub CapabilityNotSupported);

impl<D: Device, U> QueueFamily<D, U> {
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
        }
    }

    pub fn with_capability<Q: QueueCapability>(
        &self,
        _capability: Q,
    ) -> Result<QueueFamily<D, Q>, CapabilityNotSupported> {
        if self
            .queue_family_properties()
            .queue_flags
            .contains(Q::CAPABILITY)
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
