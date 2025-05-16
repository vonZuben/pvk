use crate::type_conversions::ConvertWrapper;
use crate::vk::{DeviceMemory, MappedMemory};

use vk_safe_sys as vk;

struct_wrapper!(
/// A range of memory for flushing or invalidating
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkMappedMemoryRange.html>
MappedMemoryRange<'a, S,>
);

impl<'a, S> MappedMemoryRange<'a, S> {
    /// Make a range to that represents the entire [`MappedMemory`]
    pub fn whole_range<M: DeviceMemory<Device = S>>(mapped_memory: &'a MappedMemory<M>) -> Self {
        check_vuids::check_vuids!(MappedMemoryRange);

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_memory_00684: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memory must be currently host mapped"
            }

            // ensured by MappedMemory
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_size_00685: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If size is not equal to VK_WHOLE_SIZE, offset and size must specify a range contained"
            "within the currently mapped range of memory"
            }

            // this function always sets VK_WHOLE_SIZE
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_size_00686: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If size is equal to VK_WHOLE_SIZE, offset must be within the currently mapped range"
            "of memory"
            }

            // this function always sets 0 offset
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_offset_00687: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "offset must be a multiple of VkPhysicalDeviceLimits::nonCoherentAtomSize"
            }

            // this function always sets 0 offset
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_size_01389: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If size is equal to VK_WHOLE_SIZE, the end of the current mapping of memory must either"
            "be a multiple of VkPhysicalDeviceLimits::nonCoherentAtomSize bytes from the beginning"
            "of the memory object, or be equal to the end of the memory object"
            }

            // TODO: all mappings currently map the whole range of the memory object
            // however, if this changes in future, then this needs to be considered more carefully
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_size_01390: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If size is not equal to VK_WHOLE_SIZE, size must either be a multiple of VkPhysicalDeviceLimits::nonCoherentAtomSize,"
            "or offset plus size must equal the size of memory"
            }

            // this function always sets VK_WHOLE_SIZE
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pNext must be NULL"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkMappedMemoryRange_memory_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memory must be a valid VkDeviceMemory handle"
            }

            // ensured by DeviceMemory creation
        }

        unsafe {
            Self::from_c(vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: std::ptr::null(),
                memory: mapped_memory.handle(),
                offset: 0,
                size: vk::WHOLE_SIZE,
            })
        }
    }
}
