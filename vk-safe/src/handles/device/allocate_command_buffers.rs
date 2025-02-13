use super::Device;

use crate::buffer::Buffer;
use crate::error::Error;
use crate::handles::command_buffer::{_CommandBuffers, make_command_buffers};
use crate::structs::CommandBufferAllocateInfo;

use vk_safe_sys as vk;

use vk::has_command::AllocateCommandBuffers;

pub(crate) fn allocate_command_buffers<
    'a,
    D: Device<Commands: AllocateCommandBuffers<X>>,
    Pool,
    Level,
    B: Buffer<vk::CommandBuffer>,
    X,
>(
    device: &'a D,
    alloc_info: CommandBufferAllocateInfo<'_, B, Pool, Level>,
) -> Result<_CommandBuffers<'a, D, Level, B>, Error> {
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

        // set when creating CommandBufferAllocateInfo
    }

    #[allow(unused_labels)]
    'VUID_vkAllocateCommandBuffers_pAllocateInfo_commandBufferCount_arraylength: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pAllocateInfo-&gt;commandBufferCount must be greater than 0"
        }

        // checked when creating CommandBufferAllocateInfo
    }

    let fptr = device.commands().AllocateCommandBuffers().get_fptr();

    let mut buffer = alloc_info.buffer;
    let command_buffer_count = alloc_info.info.command_buffer_count;

    unsafe {
        let res = fptr(device.raw_handle(), &alloc_info.info, buffer.ptr_mut());
        check_raw_err!(res);

        // if there is no error, then it is guaranteed that
        // command_buffer_count number of valid CommandBuffers
        // handles were written to the buffer
        buffer.set_len(
            command_buffer_count
                .try_into()
                .expect("u32 to usize should work"),
        );
    }

    Ok(make_command_buffers(device, buffer))
}
