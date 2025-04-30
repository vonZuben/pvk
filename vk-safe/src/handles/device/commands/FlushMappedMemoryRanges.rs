use crate::error::Error;
use crate::handles::DispatchableHandle;
use crate::structs::MappedMemoryRange;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

pub trait FlushMappedMemoryRanges: DispatchableHandle<RawHandle = vk::Device> {
    /// Flush memory to make host writes visible to the device
    ///
    /// ```
    /// # use vk_safe::vk;
    /// # fn tst<
    /// #    D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// #    M: vk::DeviceMemory<Device = D>
    /// # >
    /// #   (device: D, mapped_memory: vk::MappedMemory<M>) {
    /// let ranges = [vk::MappedMemoryRange::whole_range(&mapped_memory)];
    /// device.flush_mapped_memory_ranges(&ranges).unwrap();
    /// # }
    /// ```
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html>
    fn flush_mapped_memory_ranges<X>(&self, ranges: &[MappedMemoryRange<Self>]) -> Result<(), Error>
    where
        Self::Commands: vk::has_command::FlushMappedMemoryRanges<X>,
    {
        use vk::has_command::FlushMappedMemoryRanges;

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

        let fptr = self.commands().FlushMappedMemoryRanges().get_fptr();
        unsafe {
            let res = fptr(self.raw_handle(), ranges.len().try_into()?, ranges.to_c());
            check_raw_err!(res);
            Ok(())
        }
    }
}
