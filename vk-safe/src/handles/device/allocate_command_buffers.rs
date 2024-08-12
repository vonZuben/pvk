use super::Device;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::handles::command_buffer::{_CommandBuffers, make_command_buffers};
use crate::handles::command_pool::CommandPool;
use crate::structs::CommandBufferAllocateInfo;
use crate::type_conversions::SafeTransmute;

use vk_safe_sys as vk;

use vk::has_command::AllocateCommandBuffers;

unit_error!(CommandBufferCountZero);
unit_error!(StorageLenError);

pub(crate) fn allocate_command_buffers<
    'a,
    D: Device<Commands: AllocateCommandBuffers>,
    P: CommandPool,
    L,
    A: ArrayStorage<vk::CommandBuffer>,
>(
    device: &'a D,
    info: &CommandBufferAllocateInfo<'_, P, L>,
    mut storage: A,
) -> Result<_CommandBuffers<'a, D, L, A::InitStorage>, Error> {
    check_vuids::check_vuids!(AllocateCommandBuffers);

    #[allow(unused_labels)]
    'VUID_vkAllocateCommandBuffers_device_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "device must be a valid VkDevice handle"
        }

        // Device RAII
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateCommandBuffers_pAllocateInfo_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pAllocateInfo must be a valid pointer to a valid VkCommandBufferAllocateInfo structure"
        }

        // CommandBufferAllocateInfo RAII
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateCommandBuffers_pCommandBuffers_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pCommandBuffers must be a valid pointer to an array of pAllocateInfo-&gt;commandBufferCount"
        "VkCommandBuffer handles"
        }

        storage.allocate(|| Ok(info.command_buffer_count as usize))?;
        if storage.uninit_slice().len() != info.command_buffer_count as usize {
            Err(StorageLenError)?
        }
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateCommandBuffers_pAllocateInfo_commandBufferCount_arraylength: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pAllocateInfo-&gt;commandBufferCount must be greater than 0"
        }

        if info.command_buffer_count == 0 {
            Err(CommandBufferCountZero)?
        }
    }

    let array = storage.uninit_slice();
    let fptr = device.commands().AllocateCommandBuffers().get_fptr();

    unsafe {
        let res = fptr(
            device.raw_handle(),
            info.safe_transmute(),
            array.safe_transmute(),
        );
        check_raw_err!(res);
    }

    // this was checked above to ensure that the len was correct
    let len = storage.uninit_slice().len();
    let fin = storage.finalize(len);

    Ok(make_command_buffers(device, fin))
}
