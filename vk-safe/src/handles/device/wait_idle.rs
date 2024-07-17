use super::Device;

use crate::error::Error;

use vk_safe_sys as vk;

use vk::has_command::DeviceWaitIdle;

pub(crate) unsafe fn wait_idle(
    device: &impl Device<Commands: DeviceWaitIdle>,
) -> Result<(), Error> {
    let fptr = device.commands().DeviceWaitIdle().get_fptr();

    check_vuids::check_vuids!(DeviceWaitIdle);

    #[allow(unused_labels)]
    'VUID_vkDeviceWaitIdle_device_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "device must be a valid VkDevice handle"
        }

        // ensured by device creation
    }

    unsafe {
        let res = fptr(device.raw_handle());
        check_raw_err!(res);
        Ok(())
    }
}
