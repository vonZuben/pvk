use super::*;

use vk::has_command::GetDeviceQueue;
use vk::queue_capability::QueueCapability;

#[derive(Clone, Copy, Debug)]
pub struct QueueFamily<D> {
    family_index: u32,
    queue_count: u32,
    flags: vk::DeviceQueueCreateFlags,
    capabilities: vk::QueueFlags,
    device: D,
}

#[derive(Debug)]
pub struct CapabilityNotSupported;

impl std::fmt::Display for CapabilityNotSupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for CapabilityNotSupported {}

impl<D: Device> QueueFamily<D> {
    pub fn queue_scope<Q: QueueCapability, R>(
        self,
        _capability: Q,
        f: impl for<'s> FnOnce(Scope<'s, QueueFamily<D>>) -> R,
    ) -> Result<impl FnOnce() -> R, CapabilityNotSupported> {
        if self.capabilities.contains(Q::CAPABILITY) {
            Ok(move || f(Scope::new_scope(&self)))
        } else {
            Err(CapabilityNotSupported)
        }
    }
}

impl<'a, S, C: DeviceConfig> ScopedDeviceType<S, C> {
    /// get the configured queue families
    pub fn get_configured_queue_families(
        &self,
    ) -> impl Iterator<Item = QueueFamily<S>> + crate::scope::Captures<&Self> {
        self.config.queue_config().iter().map(|c| QueueFamily {
            family_index: c.queue_family_index,
            queue_count: c.queue_count,
            flags: c.flags,
            capabilities: unsafe {
                self.config
                    .queue_family_properties()
                    .get_unchecked(c.queue_family_index as usize)
                    .queue_flags
            },
            device: self.as_scope(),
        })
    }
}
