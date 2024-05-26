/*!
Unmap memory for host access

use the [`unmap_memory`](concrete_type::ScopedDevice::unmap_memory) method on a scoped Device

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkUnmapMemory.html>
*/

use super::*;

use vk_safe_sys as vk;

use vk::has_command::UnmapMemory;

use crate::non_dispatchable_handles::device_memory::{DeviceMemory, MappedMemory};

impl<S, C: concrete_type::DeviceConfig> concrete_type::ScopedDevice<S, C>
where
    C::Context: UnmapMemory,
{
    /**
    Unmap memory for host access

    ```rust
    # use vk_safe::vk;
    # fn tst<
    #    C: vk::device::VERSION_1_0,
    #    D: vk::Device<Context = C>,
    #    M: vk::DeviceMemory<Device = D>,
    # >
    #   (device: D, mapped_memory: vk::MappedMemory<M>) {
    let memory = device.unmap_memory(mapped_memory);
    # }
    ```
    */
    pub fn unmap_memory<M: DeviceMemory<Device = S>>(&self, mapped_memory: MappedMemory<M>) -> M {
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

        let fptr = self.deref().context.UnmapMemory().get_fptr();
        unsafe { fptr(self.deref().handle, mapped_memory.memory.handle) }

        mapped_memory.memory
    }
}
