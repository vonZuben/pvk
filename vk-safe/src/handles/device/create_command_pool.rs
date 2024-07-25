use super::Device;

use crate::error::Error;
use crate::handles::command_pool::{_CommandPool, make_command_pool};
use crate::structs::CommandPoolCreateInfo;
use crate::type_conversions::{SafeTransmute, ToC};

use vk_safe_sys as vk;

use vk::has_command::{CreateCommandPool, DestroyCommandPool};

pub(crate) fn create_command_pool<
    'a,
    D: Device<Commands: CreateCommandPool + DestroyCommandPool>,
    F,
    T,
>(
    device: &'a D,
    create_info: &CommandPoolCreateInfo<D, F, T>,
) -> Result<_CommandPool<'a, D, F, T>, Error> {
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
            create_info.safe_transmute(),
            None.to_c(),
            handle.as_mut_ptr(),
        );
        check_raw_err!(res);
        handle.assume_init()
    };

    Ok(make_command_pool(handle, device))
}
