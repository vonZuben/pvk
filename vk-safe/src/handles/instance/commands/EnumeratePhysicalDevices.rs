use crate::enumerator::Enumerator;
use crate::handles::physical_device::PhysicalDeviceHandle;
use crate::handles::DispatchableHandle;
use crate::scope::Captures;

use vk_safe_sys as vk;

pub trait EnumeratePhysicalDevices: DispatchableHandle<RawHandle = vk::Instance> {
    /// Enumerate PhysicalDevices on the system
    ///
    /// # Usage
    /// Use the resulting [`Enumerator`] to retrieve an array of [`PhysicalDeviceHandle`].
    /// Then you can iterate over the handles and tag each one that
    /// you want to use with a [`Tag`].
    ///
    /// # Example
    /// ```
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst(instance: impl Instance<Commands: vk::instance::VERSION_1_0>) {
    /// let physical_devices = instance
    ///     .enumerate_physical_devices()
    ///     .auto_get_enumerate()
    ///     .unwrap();
    ///
    /// for physical_device in physical_devices.iter() {
    ///     vk::tag!(tag);
    ///     let physical_device = physical_device.tag(&instance, tag);
    /// }
    /// # }
    /// ```
    ///
    /// Vulkan docs:
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html>
    fn enumerate_physical_devices<'a, X>(
        &'a self,
    ) -> impl Enumerator<PhysicalDeviceHandle<Self>> + Captures<&'a Self>
    where
        Self::Commands: vk::has_command::EnumeratePhysicalDevices<X>,
    {
        use vk::has_command::EnumeratePhysicalDevices;

        check_vuids::check_vuids!(EnumeratePhysicalDevices);

        #[allow(unused_labels)]
        'VUID_vkEnumeratePhysicalDevices_instance_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "instance must be a valid VkInstance handle"
            }

            // always valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkEnumeratePhysicalDevices_pPhysicalDeviceCount_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pPhysicalDeviceCount must be a valid pointer to a uint32_t value"
            }

            // enumerator_code2!
        }

        #[allow(unused_labels)]
        'VUID_vkEnumeratePhysicalDevices_pPhysicalDevices_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the value referenced by pPhysicalDeviceCount is not 0, and pPhysicalDevices is"
            "not NULL, pPhysicalDevices must be a valid pointer to an array of pPhysicalDeviceCount"
            "VkPhysicalDevice handles"
            }

            //enumerator_code2!
        }

        make_enumerator!(self.commands().EnumeratePhysicalDevices().get_fptr(); (self.raw_handle()))
    }
}
