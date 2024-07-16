use super::{Device, DeviceMemory, MappedMemory};

use vk_safe_sys as vk;

use vk::has_command::UnmapMemory;

pub(crate) fn unmap_memory<D: Device<Commands: UnmapMemory>, M: DeviceMemory<Device = D>>(
    device: &D,
    mapped_memory: MappedMemory<M>,
) -> M {
    check_vuids::check_vuids!(UnmapMemory);

    #[allow(unused_labels)]
    'VUID_vkUnmapMemory_memory_00689: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "memory must be currently host mapped"
        }

        // MappedMemory can only be created by mapping the memory
    }

    #[allow(unused_labels)]
    'VUID_vkUnmapMemory_device_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "device must be a valid VkDevice handle"
        }

        // ensured by device creation
    }

    #[allow(unused_labels)]
    'VUID_vkUnmapMemory_memory_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "memory must be a valid VkDeviceMemory handle"
        }

        // ensured by memory allocation
    }

    #[allow(unused_labels)]
    'VUID_vkUnmapMemory_memory_parent: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "memory must have been created, allocated, or retrieved from device"
        }

        // ensured by Device = S
    }

    let fptr = device.commands().UnmapMemory().get_fptr();
    unsafe {
        fptr(device.raw_handle(), mapped_memory.handle());
        mapped_memory.take()
    }
}
