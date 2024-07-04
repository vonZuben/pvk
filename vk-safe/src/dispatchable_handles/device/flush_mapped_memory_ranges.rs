//! Flush memory so host writes are visible to the device
//!
//! When a host writes data to non-coherent memory, it is not
//! guaranteed to be visible to the device until it is flushed.
//!
//! use [`flush_mapped_memory_ranges`](ScopedDevice::flush_mapped_memory_ranges) to flush
//! the memory to be visible to the device.
//!
//! Vulkan docs:
//! <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html>

use super::concrete_type::{DeviceConfig, ScopedDevice};
use vk_safe_sys as vk;

use std::marker::PhantomData;

use crate::error::Error;
use crate::non_dispatchable_handles::device_memory::{DeviceMemory, MappedMemory};
use crate::type_conversions::SafeTransmute;

use vk::has_command::FlushMappedMemoryRanges;

impl<S, C: DeviceConfig> ScopedDevice<S, C>
where
    C::Context: FlushMappedMemoryRanges,
{
    /// Flush memory to make host writes visible to the device
    ///
    /// ```
    /// # use vk_safe::vk;
    /// # fn tst<
    /// #    D: vk::Device<Context: vk::device::VERSION_1_0>,
    /// #    M: vk::DeviceMemory<Device = D>
    /// # >
    /// #   (device: D, mapped_memory: vk::MappedMemory<M>) {
    /// let ranges = [vk::MappedMemoryRange::whole_range(&mapped_memory)];
    /// device.flush_mapped_memory_ranges(&ranges).unwrap();
    /// # }
    /// ```
    pub fn flush_mapped_memory_ranges(&self, ranges: &[MappedMemoryRange<S>]) -> Result<(), Error> {
        check_vuids::check_vuids!(FlushMappedMemoryRanges);

        #[allow(unused_labels)]
        'VUID_vkFlushMappedMemoryRanges_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "device must be a valid VkDevice handle"
            }

            // ensured by device creation
        }

        #[allow(unused_labels)]
        'VUID_vkFlushMappedMemoryRanges_pMemoryRanges_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pMemoryRanges must be a valid pointer to an array of memoryRangeCount valid VkMappedMemoryRange"
            "structures"
            }

            // ensured by &[MappedMemoryRange<S>]
        }

        #[allow(unused_labels)]
        'VUID_vkFlushMappedMemoryRanges_memoryRangeCount_arraylength: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memoryRangeCount must be greater than 0"
            }

            // use &[MappedMemoryRange<S>] len()
        }

        let fptr = self.deref().context.FlushMappedMemoryRanges().get_fptr();
        unsafe {
            let res = fptr(
                self.deref().handle,
                ranges.len().try_into()?,
                ranges.safe_transmute().as_ptr(),
            );
            check_raw_err!(res);
            Ok(())
        }
    }
}

input_struct_wrapper!(
/// A range of memory for flushing or invalidating
MappedMemoryRange
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

        Self {
            inner: vk_safe_sys::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: std::ptr::null(),
                memory: mapped_memory.memory.handle,
                offset: 0,
                size: vk::WHOLE_SIZE,
            },
            _params: PhantomData,
            _scope: PhantomData,
        }
    }
}
