use crate::handles::DispatchableHandle;

use crate::error::Error;

use vk_safe_sys as vk;

pub trait DeviceWaitIdle: DispatchableHandle<RawHandle = vk::Device> {
    /// Wait for all queue operations on the device to complete.
    ///
    /// Blocks until **all** operations on **all** `Queue`s belonging to this `Device` are
    /// complete.
    ///
    /// *Can fail in exceptional situations. Will return Ok(()) on success.*
    ///
    /// # SAFETY
    /// You **must not** call any methods on any [`Queue`](crate::vk::Queue) object
    /// created from this Device, on any other threads at the same time as calling
    /// this method.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # fn tst<
    /// #    D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// # >
    /// #   (mut device: D) {
    /// let result = unsafe { device.wait_idle() };
    /// # }
    /// ```
    unsafe fn wait_idle<X>(&self) -> Result<(), Error>
    where
        Self::Commands: vk::has_command::DeviceWaitIdle<X>,
    {
        use vk::has_command::DeviceWaitIdle;

        let fptr = self.commands().DeviceWaitIdle().get_fptr();

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
            let res = fptr(self.raw_handle());
            check_raw_err!(res);
            Ok(())
        }
    }
}
