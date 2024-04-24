use super::{DeviceConfig, ScopedDeviceType};
use crate::error::Error;

use vk_safe_sys as vk;

use vk::has_command::DeviceWaitIdle;

impl<S, C: DeviceConfig> ScopedDeviceType<S, C>
where
    C::Context: DeviceWaitIdle,
{
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDeviceWaitIdle.html>
    pub fn wait_idle(&self) -> Result<(), Error> {
        let fptr = self.context.DeviceWaitIdle().get_fptr();
        unsafe {
            check_vuids::check_vuids!(DeviceWaitIdle);

            #[allow(unused_labels)]
            'VUID_vkDeviceWaitIdle_device_parameter: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "device must be a valid VkDevice handle"
                }

                // ensured by device creation
            }

            let res = fptr(self.deref().handle);
            check_raw_err!(res);
            Ok(())
        }
    }
}
