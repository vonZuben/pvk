use std::fmt;

use super::QueueFamilyProperties;

use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

struct_wrapper!(
/// Info struct for creating DeviceQueues
///
/// When creating a [`Device`](crate::vk::Device), this struct provides
/// information about the Queues to be created therewith.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDeviceQueueCreateInfo.html>
DeviceQueueCreateInfo<'a, Z,>
impl Deref
);

impl<Z> fmt::Debug for DeviceQueueCreateInfo<'_, Z> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceQueueCreateInfo")
            .field("flags", &self.inner.flags)
            .field("p_next", &"TODO")
            .field("queue_family_index", &self.inner.queue_family_index)
            .field("queue_count", &self.inner.queue_count)
            .field("queue_priorities", &self.queue_priorities())
            .finish()
    }
}

enum_error!(
    pub enum DeviceQueueCreateInfoError {
        TooManyQueues,
        ZeroIsInvalid,
    }
);

impl<'a, Z> DeviceQueueCreateInfo<'a, Z> {
    array!(queue_priorities, p_queue_priorities, queue_count, f32);

    /// Create DeviceQueueCreateInfo
    ///
    /// When creating a [`Device`](crate::vk::Device), create
    /// `priorities.len()` number of Queues, with respective
    /// priorities. **Must** create at least one Queue.
    pub fn new(
        priorities: &'a [QueuePriority],
        family: QueueFamilyProperties<Z>,
    ) -> Result<Self, DeviceQueueCreateInfoError> {
        check_vuids::check_vuids!(DeviceQueueCreateInfo);

        let priorities_len: u32 = priorities
            .len()
            .try_into()
            .map_err(|_| DeviceQueueCreateInfoError::TooManyQueues)?;

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueFamilyIndex must be less than pQueueFamilyPropertyCount returned by vkGetPhysicalDeviceQueueFamilyProperties"
            }

            // QueueFamily invariant
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_00382: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueCount must be less than or equal to the queueCount member of the VkQueueFamilyProperties"
            "structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties in the pQueueFamilyProperties[queueFamilyIndex]"
            }

            if priorities_len > family.queue_count {
                Err(DeviceQueueCreateInfoError::TooManyQueues)?
            }
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "Each element of pQueuePriorities must be between 0.0 and 1.0 inclusive"
            }

            // [Priority]
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_02861: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the protectedMemory feature is not enabled, the VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT"
            "bit of flags must not be set"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_06449: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If flags includes VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT, queueFamilyIndex must be the"
            "index of a queue family that includes the VK_QUEUE_PROTECTED_BIT capability"
            }

            // flags not supported for now
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pNext must be NULL or a pointer to a valid instance of VkDeviceQueueGlobalPriorityCreateInfoKHR"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_unique: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "The sType value of each struct in the pNext chain must be unique"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be a valid combination of VkDeviceQueueCreateFlagBits values"
            }

            // vk::DeviceQueueCreateFlags, and checking VUID_VkDeviceQueueCreateInfo_flags_06449 above
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pQueuePriorities must be a valid pointer to an array of queueCount float values"
            }

            // rust reference, and `Priority` is #[repr(transparent)] f32
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueCount must be greater than 0"
            }

            if priorities.len() == 0 {
                Err(DeviceQueueCreateInfoError::ZeroIsInvalid)?
            }
        }

        Ok(unsafe {
            Self::from_c(vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: family.family_index,
                queue_count: priorities_len,
                p_queue_priorities: priorities.to_c(),
            })
        })
    }
}

unit_error!(pub InvalidPriorityValue);

/// Priority of a Queue
///
/// Always has a value between 0.0 and 1.0 (inclusive)
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct QueuePriority {
    priority: f32,
}

unsafe impl crate::type_conversions::ConvertWrapper<f32> for QueuePriority {}

impl QueuePriority {
    /// Create a new Priority
    ///
    /// Will create a new priority with a value between 0.0 and 1.0 (inclusive)
    ///
    /// Will return an error if the provided value is outside the allowed range
    pub fn new(priority: f32) -> Result<Self, InvalidPriorityValue> {
        match priority {
            0.0..=1.0 => Ok(Self { priority }),
            _ => Err(InvalidPriorityValue),
        }
    }

    /// Create a new Priority without checking that it has a valid value
    ///
    /// The caller must ensure that the priority is between 0.0 and 1.0 (inclusive)
    pub unsafe fn new_unchecked(priority: f32) -> Self {
        Self { priority }
    }
}

impl std::default::Default for QueuePriority {
    /// Returns a Priority of 0.0
    fn default() -> Self {
        Self {
            priority: Default::default(),
        }
    }
}
