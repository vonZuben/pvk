use std::marker::PhantomData;
use std::ops::Deref;

use crate::type_conversions::SafeTransmute;
use crate::vk::CommandPool;

use vk_safe_sys as vk;

/// Info for allocating CommandBuffers
///
/// Indicates which pool to allocate from, the level of the
/// CommandBuffers, and how many to allocate.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCommandBufferAllocateInfo.html>
#[repr(transparent)]
pub struct CommandBufferAllocateInfo<'a, P, L> {
    inner: vk::CommandBufferAllocateInfo,
    pool: PhantomData<&'a P>,
    level: PhantomData<L>,
}

unsafe impl<P, L> SafeTransmute<vk::CommandBufferAllocateInfo>
    for CommandBufferAllocateInfo<'_, P, L>
{
}

impl<'a, P, L> Deref for CommandBufferAllocateInfo<'a, P, L> {
    type Target = vk::CommandBufferAllocateInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, P: CommandPool, L: vk::enum_traits::CommandBufferLevel>
    CommandBufferAllocateInfo<'a, P, L>
{
    pub fn new(command_pool: &'a P, level: L, command_buffer_count: u32) -> Self {
        check_vuids::check_vuids!(CommandBufferAllocateInfo);

        #[allow(unused_labels)]
        'VUID_VkCommandBufferAllocateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO"
            }

            // set bellow
        }

        #[allow(unused_labels)]
        'VUID_VkCommandBufferAllocateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pNext must be NULL"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkCommandBufferAllocateInfo_commandPool_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "commandPool must be a valid VkCommandPool handle"
            }

            // ensured by CommandPool
        }

        #[allow(unused_labels)]
        'VUID_VkCommandBufferAllocateInfo_level_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "level must be a valid VkCommandBufferLevel value"
            }

            // Flags<Type = vk::CommandBufferLevel>>
        }

        let _ = level;

        Self {
            inner: vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                p_next: std::ptr::null(),
                command_pool: command_pool.raw_handle(),
                level: L::VALUE,
                command_buffer_count,
            },
            pool: PhantomData,
            level: PhantomData,
        }
    }
}
