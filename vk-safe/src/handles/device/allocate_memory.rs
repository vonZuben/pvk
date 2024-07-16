use super::Device;

use std::mem::MaybeUninit;

use crate::flags::Flags;
use crate::structs::MemoryAllocateInfo;
use crate::type_conversions::SafeTransmute;
use crate::vk::_DeviceMemory;

use vk::has_command::{AllocateMemory, FreeMemory};
use vk_safe_sys as vk;

pub(crate) fn allocate_memory<
    'a,
    D: Device<Commands: AllocateMemory + FreeMemory>,
    P: Flags,
    H: Flags,
>(
    device: &'a D,
    info: &MemoryAllocateInfo<D::PhysicalDevice, P, H>,
) -> Result<
    // impl DeviceMemory<Device = S, PropertyFlags = P, HeapFlags = H> + Captures<&Self>,
    _DeviceMemory<'a, D, P, H>,
    vk::Result,
> {
    check_vuids::check_vuids!(AllocateMemory);

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_pAllocateInfo_01713: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pAllocateInfo-&gt;allocationSize must be less than or equal to VkPhysicalDeviceMemoryProperties::memoryHeaps[memindex].size"
        "where memindex = VkPhysicalDeviceMemoryProperties::memoryTypes[pAllocateInfo-&gt;memoryTypeIndex].heapIndex"
        "as returned by vkGetPhysicalDeviceMemoryProperties for the VkPhysicalDevice that device"
        "was created from"
        }

        // ******************** TODO ****************************
        // this is not currently checked at all
        // probably unusual to go over this limit in a single allocation these days
        // still I will need to add a check for this somehow
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_pAllocateInfo_01714: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pAllocateInfo-&gt;memoryTypeIndex must be less than VkPhysicalDeviceMemoryProperties::memoryTypeCount"
        "as returned by vkGetPhysicalDeviceMemoryProperties for the VkPhysicalDevice that device"
        "was created from"
        }

        // ensured by MemoryTypeChoice in MemoryAllocateInfo
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_deviceCoherentMemory_02790: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the deviceCoherentMemory feature is not enabled, pAllocateInfo-&gt;memoryTypeIndex"
        "must not identify a memory type supporting VK_MEMORY_PROPERTY_DEVICE_COHERENT_BIT_AMD"
        }

        // ensured by MemoryTypeChoice in MemoryAllocateInfo
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_maxMemoryAllocationCount_04101: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "There must be less than VkPhysicalDeviceLimits::maxMemoryAllocationCount device memory"
        "allocations currently allocated on the device"
        }

        // ******************** TODO ****************************
        // this is not currently checked at all
        // probably unusual to go over this limit if using good memory allocation practice
        // still I will need to add a check for this somehow
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_device_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "device must be a valid VkDevice handle"
        }

        // ensured by Device creation
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_pAllocateInfo_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pAllocateInfo must be a valid pointer to a valid VkMemoryAllocateInfo structure"
        }

        // ensured by MemoryAllocateInfo creation
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_pAllocator_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
        "structure"
        }

        // TODO pAllocator not currently supported
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateMemory_pMemory_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pMemory must be a valid pointer to a VkDeviceMemory handle"
        }

        // MaybeUninit
    }

    let fptr = device.commands().AllocateMemory().get_fptr();
    let mut memory = MaybeUninit::uninit();
    unsafe {
        let ret = fptr(
            device.raw_handle(),
            info.safe_transmute(),
            std::ptr::null(),
            memory.as_mut_ptr(),
        );
        check_raw_err!(ret);
        Ok(_DeviceMemory::new(memory.assume_init(), device))
    }
}
