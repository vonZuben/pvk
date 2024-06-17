/*!
Wait for all queue operations on the device to complete

use the [`wait_idle`](concrete_type::ScopedDevice::wait_idle) method on a scoped Device

Vulkan docs:
<https://registry.khronos.org/VulkanSC/specs/1.0-extensions/man/html/vkDeviceWaitIdle.html>
*/

use super::*;
use crate::error::Error;

use vk_safe_sys as vk;

use vk::has_command::DeviceWaitIdle;

impl<S, C: concrete_type::DeviceConfig> concrete_type::ScopedDevice<S, C>
where
    C::Context: DeviceWaitIdle,
{
    /**
    Wait for all queue operations on the device to complete.

    Blocks until **all** operations on **all** `Queue`s belonging to this `Device` are
    complete.

    *Can fail in exceptional situations. Will return Ok(()) on success.*

    # SAFETY
    You **must not** call and methods on any [`Queue`](crate::vk::Queue) object
    created from this Device, on any other threads at the same time as calling
    this method.

    ```rust
    # use vk_safe::vk;
    # fn tst<
    #    C: vk::device::VERSION_1_0,
    #    D: vk::Device<Context = C>,
    # >
    #   (mut device: D) {
    let result = unsafe { device.wait_idle() };
    # }
    ```
    */
    pub unsafe fn wait_idle(&self) -> Result<(), Error> {
        let fptr = self.context.DeviceWaitIdle().get_fptr();

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
            let res = fptr(self.deref().handle);
            check_raw_err!(res);
            Ok(())
        }
    }
}
