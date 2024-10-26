use std::marker::PhantomData;
use std::ops::Deref;

use crate::buffer::Buffer;
use crate::error::Error;
use crate::vk::CommandPool;

use vk_safe_sys as vk;

/// Info for allocating CommandBuffers
///
/// Indicates which pool to allocate from, the level of the
/// CommandBuffers, and how many to allocate.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCommandBufferAllocateInfo.html>
pub struct CommandBufferAllocateInfo<'a, B, P, L> {
    pub(crate) info: vk::CommandBufferAllocateInfo,
    pub(crate) buffer: B,
    pool: PhantomData<&'a P>,
    level: PhantomData<L>,
}

impl<'a, B, P, L> Deref for CommandBufferAllocateInfo<'a, B, P, L> {
    type Target = vk::CommandBufferAllocateInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

unit_error!(ZeroSizedBuffer);

impl<'a, B: Buffer<vk::CommandBuffer>, P: CommandPool, L: vk::enum_traits::CommandBufferLevel>
    CommandBufferAllocateInfo<'a, B, P, L>
{
    /// Create CommandBufferAllocateInfo
    ///
    /// The CommandBufferAllocateInfo will contain information
    /// for allocating `buffer.capacity()` number of [`CommandBuffer`],
    /// for the indicated `level`.
    pub fn new(command_pool: &'a P, buffer: B, level: L) -> Result<Self, Error> {
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

        // in relation to VUID_vkAllocateCommandBuffers_pCommandBuffers_parameter and VUID_vkAllocateCommandBuffers_pAllocateInfo_commandBufferCount_arraylength
        // commandBufferCount is set based on the buffer
        // we check that the buffer capacity is > 0
        if buffer.capacity() == 0 {
            Err(ZeroSizedBuffer)?
        }

        let _ = level;

        Ok(Self {
            info: vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                p_next: std::ptr::null(),
                command_pool: command_pool.raw_handle(),
                level: L::VALUE,
                command_buffer_count: buffer.capacity().try_into()?,
            },
            buffer,
            pool: PhantomData,
            level: PhantomData,
        })
    }
}
