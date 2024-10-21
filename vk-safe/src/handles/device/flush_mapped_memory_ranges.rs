use super::Device;

use crate::error::Error;
use crate::structs::MappedMemoryRange;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::has_command::FlushMappedMemoryRanges;

pub(crate) fn flush_mapped_memory_ranges<D: Device<Commands: FlushMappedMemoryRanges>>(
    device: &D,
    ranges: &[MappedMemoryRange<D>],
) -> Result<(), Error> {
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

    let fptr = device.commands().FlushMappedMemoryRanges().get_fptr();
    unsafe {
        let res = fptr(device.raw_handle(), ranges.len().try_into()?, ranges.to_c());
        check_raw_err!(res);
        Ok(())
    }
}
