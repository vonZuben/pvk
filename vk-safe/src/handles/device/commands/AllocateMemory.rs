use std::mem::MaybeUninit;

use crate::handles::DispatchableHandle;
use crate::structs::MemoryAllocateInfo;
use crate::type_conversions::ConvertWrapper;
use crate::vk::DeviceAttributes;
use crate::vk::{make_device_memory, DeviceMemory};

use vk_safe_sys as vk;

pub trait AllocateMemory: DispatchableHandle<RawHandle = vk::Device> + DeviceAttributes {
    /// Allocate memory on the Device
    ///
    /// Provide a [`MemoryAllocateInfo`] structure with the information
    /// about amount and type of memory you want to allocate.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<
    /// #   D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// #   P: vk::flag_traits::MemoryPropertyFlags,
    /// #   H: vk::flag_traits::MemoryHeapFlags,
    /// # >
    /// #   (device: D, alloc_info: vk::MemoryAllocateInfo<D::PhysicalDevice, P, H>) {
    /// let memory = device.allocate_memory(&alloc_info);
    /// # }
    /// ```
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkAllocateMemory.html>
    fn allocate_memory<
        'a,
        P: vk::flag_traits::MemoryPropertyFlags,
        H: vk::flag_traits::MemoryHeapFlags,
        X,
        Y,
    >(
        &'a self,
        info: &MemoryAllocateInfo<Self::PhysicalDevice, P, H>,
    ) -> Result<
        impl DeviceMemory<Device = Self, PropertyFlags = P, HeapFlags = H> + use<'a, Self, P, H, X, Y>,
        vk::Result,
    >
    where
        Self::Commands: vk::has_command::AllocateMemory<X> + vk::has_command::FreeMemory<Y>,
    {
        use vk::has_command::AllocateMemory;

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

        let fptr = self.commands().AllocateMemory().get_fptr();
        let mut memory = MaybeUninit::uninit();
        unsafe {
            let ret = fptr(
                self.raw_handle(),
                info.to_c(),
                std::ptr::null(),
                memory.as_mut_ptr(),
            );
            check_raw_err!(ret);
            Ok(make_device_memory(memory.assume_init(), self))
        }
    }
}
