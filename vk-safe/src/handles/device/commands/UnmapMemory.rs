use crate::handles::DispatchableHandle;
use crate::vk::{DeviceMemory, MappedMemory};

use vk_safe_sys as vk;

pub trait UnmapMemory: DispatchableHandle<RawHandle = vk::Device> {
    /// Unmap memory for host access
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # fn tst<
    /// #    D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// #    M: vk::DeviceMemory<Device = D>,
    /// # >
    /// #   (device: D, mapped_memory: vk::MappedMemory<M>) {
    /// let memory = device.unmap_memory(mapped_memory);
    /// # }
    /// ```
    fn unmap_memory<M: DeviceMemory<Device = Self>, X>(&self, mapped_memory: MappedMemory<M>) -> M
    where
        Self::Commands: vk::has_command::UnmapMemory<X>,
    {
        use vk::has_command::UnmapMemory;

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

        let fptr = self.commands().UnmapMemory().get_fptr();
        unsafe {
            fptr(self.raw_handle(), mapped_memory.handle());
            mapped_memory.take()
        }
    }
}
