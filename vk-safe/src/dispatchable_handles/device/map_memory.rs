/*!
Map memory for host access

use the [`map_memory`](concrete_type::ScopedDevice::map_memory) method on a scoped Device

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html>
*/

use super::*;
use vk_safe_sys as vk;

use crate::error::Error;

use crate::non_dispatchable_handles::device_memory::{DeviceMemory, MappedMemory};

use crate::flags::{Excludes, Includes};

use vk::flag_types::MemoryHeapFlags::MULTI_INSTANCE_BIT;
use vk::flag_types::MemoryPropertyFlags::HOST_VISIBLE_BIT;
use vk::has_command::MapMemory;

impl<D, C: concrete_type::DeviceConfig> concrete_type::ScopedDevice<D, C>
where
    C::Context: MapMemory,
{
    /**
    Map memory for host access

    ```rust
    # use vk_safe::vk;
    # vk::device_context!(D: VERSION_1_0);
    # use vk::flag_types::MemoryHeapFlags::MULTI_INSTANCE_BIT;
    # use vk::flag_types::MemoryPropertyFlags::HOST_VISIBLE_BIT;
    # fn tst<
    #    C: vk::device::VERSION_1_0,
    #    D: vk::Device<Context = C>,
    #    P: vk::Includes<HOST_VISIBLE_BIT>,
    #    H: vk::Excludes<MULTI_INSTANCE_BIT>
    # >
    #   (device: D, memory: impl vk::DeviceMemory<Device = D, PropertyFlags = P, HeapFlags = H>) {
    let mapped_memory = device.map_memory(memory);
    # }
    ```

    ### Note
    *currently this can only be used to map the whole memory range. There may be breaking change in
    future to make the API more inline with the real `vkMapMemory`, which allows mapping sub ranges*
    */
    pub fn map_memory<
        P: Includes<HOST_VISIBLE_BIT>,
        H: Excludes<MULTI_INSTANCE_BIT>,
        M: DeviceMemory<Device = D, PropertyFlags = P, HeapFlags = H>,
    >(
        &self,
        memory: M,
    ) -> Result<MappedMemory<M>, Error> {
        check_vuids::check_vuids!(MapMemory);

        #[allow(unused_labels)]
        'VUID_vkMapMemory_memory_00678: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memory must not be currently host mapped"
            }

            // memory is moved into MappedMemory, which does not allow mapping again
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_offset_00679: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "offset must be less than the size of memory"
            }

            // currently always set to zero
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_size_00680: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If size is not equal to VK_WHOLE_SIZE, size must be greater than 0"
            }

            // currently always set to VK_WHOLE_SIZE
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_size_00681: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If size is not equal to VK_WHOLE_SIZE, size must be less than or equal to the size"
            "of the memory minus offset"
            }

            // currently always set to VK_WHOLE_SIZE
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_memory_00682: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memory must have been created with a memory type that reports VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT"
            }

            // P: Flag<HOST_VISIBLE_BIT>
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_memory_00683: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memory must not have been allocated with multiple instances"
            }

            // one way for memory to be multi instance is to allocate it from a multi instance Heap
            // We ensure this is not the case with the below bound
            // H: NotFlag<MULTI_INSTANCE_BIT>
            //
            // Another way for memory to be multi instance is to be allocated with specific flags in VkMemoryAllocateFlagsInfo
            // as part of the p_next chain in VkMemoryAllocateInfo.
            // ############### TODO ###################
            // p_next is not currently supported
            // need to ensure support for p_next does not allow violating the above
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "device must be a valid VkDevice handle"
            }

            // ensured by device creation
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_memory_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memory must be a valid VkDeviceMemory handle"
            }

            // ensured by memory allocation
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_flags_zerobitmask: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be 0"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_ppData_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "ppData must be a valid pointer to a pointer value"
            }

            // MaybeUninit
        }

        #[allow(unused_labels)]
        'VUID_vkMapMemory_memory_parent: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "memory must have been created, allocated, or retrieved from device"
            }

            // ensured by Scope
        }

        let mut ptr = std::mem::MaybeUninit::uninit();

        let fptr = self.deref().context.MapMemory().get_fptr();
        unsafe {
            let res = fptr(
                self.deref().handle,
                memory.handle,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
                ptr.as_mut_ptr(),
            );
            check_raw_err!(res);

            Ok(MappedMemory::new(memory, ptr.assume_init()))
        }
    }
}
