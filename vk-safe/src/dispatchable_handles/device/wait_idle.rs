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

    # TODO
    For this method, host access to all VkQueue objects created from device must be externally synchronized.
    Use of `&mut self` ensures synchronization. However, it also makes this method
    essentially useless. This is because all child object have phantom references to the parent.
    Thus, it is only possible to wait_idle on the Device if all child objects are dropped,
    forgotten, or just never used again. It's not even possible for a user to use a Mutex
    or the like since the children are internally holding the phantom references to the parent.
    On one hand, I think it is good to encourage better idle waiting, by using semaphores, fences, or
    waiting on individual queue, which should still be possible since they are leaf child objects.
    *However, it may be best if this is an unsafe method to at least let the user have the option.*

    ```rust
    # use vk_safe::vk;
    # fn tst<
    #    C: vk::device::VERSION_1_0,
    #    D: vk::Device<Context = C>,
    # >
    #   (mut device: D) {
    let result = device.wait_idle();
    # }
    ```
    */
    pub fn wait_idle(&mut self) -> Result<(), Error> {
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
