use std::marker::PhantomData;

use crate::type_conversions::ConvertWrapper;
use crate::vk::QueueFamily;

use vk_safe_sys as vk;

use vk::flag_traits::CommandPoolCreateFlags;

/// Info for creating a CommandPool
///
/// used with [`create_command_pool`](crate::vk::Device::create_command_pool)
///
/// Must use the [`flags!`](crate::flags!) macro to declare the flags
/// that will be used with the CommandPool.
///
/// CommandBuffers form the CommandPool will be usable with
/// Queues from the provided [`QueueFamily`].
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCommandPoolCreateInfo.html>
#[repr(transparent)]
pub struct CommandPoolCreateInfo<D, F, T> {
    inner: vk::CommandPoolCreateInfo,
    device: PhantomData<D>,
    flags: PhantomData<F>,
    queue_family_tag: PhantomData<T>,
}

unsafe impl<D, F, T> ConvertWrapper<vk::CommandPoolCreateInfo>
    for CommandPoolCreateInfo<D, F, T>
{
}

impl<D, F, T> std::ops::Deref for CommandPoolCreateInfo<D, F, T> {
    type Target = vk::CommandPoolCreateInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D, F: CommandPoolCreateFlags, T> CommandPoolCreateInfo<D, F, T> {
    pub fn new<'a>(flags: F, queue_family: &impl QueueFamily<'a, Device = D, Tag = T>) -> Self {
        check_vuids::check_vuids!(CommandPoolCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkCommandPoolCreateInfo_flags_02860: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the protectedMemory feature is not enabled, the VK_COMMAND_POOL_CREATE_PROTECTED_BIT"
            "bit of flags must not be set"
            }

            // ********************TODO*********************
            // need to add support for Features as types??
            // simply reject for now
            const {
                if F::INCLUDES.contains(vk::CommandPoolCreateFlags::PROTECTED_BIT) {
                    panic!("PROTECTED_BIT not supported by vk-safe at this time")
                }
            }
        }

        #[allow(unused_labels)]
        'VUID_VkCommandPoolCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkCommandPoolCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pNext must be NULL"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkCommandPoolCreateInfo_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be a valid combination of VkCommandPoolCreateFlagBits values"
            }

            // ensured by CommandPoolCreateFlags type
        }

        let _ = flags; // just used for the type
        Self {
            inner: vk::CommandPoolCreateInfo {
                s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: F::INCLUDES,
                queue_family_index: queue_family.family_index(),
            },
            device: PhantomData,
            flags: PhantomData,
            queue_family_tag: PhantomData,
        }
    }
}
