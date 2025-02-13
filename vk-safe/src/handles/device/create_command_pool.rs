use super::Device;

use crate::error::Error;
use crate::handles::command_pool::{make_command_pool, CommandPool};
use crate::structs::CommandPoolCreateInfo;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::flag_traits::CommandPoolCreateFlags;
use vk::has_command::{CreateCommandPool, DestroyCommandPool};

pub fn create_command_pool<
    'a,
    D: Device<Commands: CreateCommandPool<X> + DestroyCommandPool<Y>>,
    F: CommandPoolCreateFlags,
    Q: Send,
    X,
    Y,
>(
    device: &'a D,
    create_info: &CommandPoolCreateInfo<D, F, Q>,
) -> Result<impl CommandPool<Device = D, Flags = F, QueueFamily = Q> + use<'a, D, F, Q, X, Y>, Error>
{
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
    let fptr = device.commands().CreateCommandPool().get_fptr();
    let handle = unsafe {
        let res = fptr(
            device.raw_handle(),
            create_info.to_c(),
            std::ptr::null(),
            handle.as_mut_ptr(),
        );
        check_raw_err!(res);
        handle.assume_init()
    };

    Ok(make_command_pool(handle, device))
}
