use crate::error::Error;
use crate::handles::command_pool::{make_command_pool, CommandPool};
use crate::handles::DispatchableHandle;
use crate::structs::CommandPoolCreateInfo;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::flag_traits::CommandPoolCreateFlags;

pub trait CreateCommandPool: DispatchableHandle<RawHandle = vk::Device> {
    /// Create a CommandPool
    ///
    /// ```
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<'a, D: Device<Commands: vk::device::VERSION_1_0>>
    /// #   (device: D, queue_family: impl vk::QueueFamily<'a, Device = D>) {
    /// vk::flags!(CPflags: CommandPoolCreateFlags + RESET_COMMAND_BUFFER_BIT);
    /// let command_pool = device
    /// .create_command_pool(&vk::CommandPoolCreateInfo::new(CPflags, &queue_family))
    /// .unwrap();
    /// # }
    /// ```
    fn create_command_pool<'a, F: CommandPoolCreateFlags, Q: Send, X, Y>(
        &'a self,
        create_info: &CommandPoolCreateInfo<Self, F, Q>,
    ) -> Result<
        impl CommandPool<Device = Self, Flags = F, QueueFamily = Q> + use<'a, Self, F, Q, X, Y>,
        Error,
    >
    where
        Self::Commands:
            vk::has_command::CreateCommandPool<X> + vk::has_command::DestroyCommandPool<Y>,
    {
        use vk::has_command::CreateCommandPool;

        check_vuids::check_vuids!(CreateCommandPool);

        #[allow(unused_labels)]
        'VUID_vkCreateCommandPool_queueFamilyIndex_01937: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pCreateInfo-&gt;queueFamilyIndex must be the index of a queue family available in"
            "the logical device device"
            }

            // CommandPoolCreateInfo is created from a QueueFamily associated with the Device D
        }

        #[allow(unused_labels)]
        'VUID_vkCreateCommandPool_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "device must be a valid VkDevice handle"
            }

            // ensured by device creation
        }

        #[allow(unused_labels)]
        'VUID_vkCreateCommandPool_pCreateInfo_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pCreateInfo must be a valid pointer to a valid VkCommandPoolCreateInfo structure"
            }

            // ensured by CommandPoolCreateInfo creation
        }

        #[allow(unused_labels)]
        'VUID_vkCreateCommandPool_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO
            // always null for now
        }

        #[allow(unused_labels)]
        'VUID_vkCreateCommandPool_pCommandPool_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pCommandPool must be a valid pointer to a VkCommandPool handle"
            }

            // MaybeUninit
        }

        let mut handle = std::mem::MaybeUninit::uninit();
        let fptr = self.commands().CreateCommandPool().get_fptr();
        let handle = unsafe {
            let res = fptr(
                self.raw_handle(),
                create_info.to_c(),
                std::ptr::null(),
                handle.as_mut_ptr(),
            );
            check_raw_err!(res);
            handle.assume_init()
        };

        Ok(make_command_pool(handle, self))
    }
}
