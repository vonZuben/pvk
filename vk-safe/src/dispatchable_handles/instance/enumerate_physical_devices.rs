/*!
enumerate physical devices on the system

After creating a scoped [`Instance`], you can enumerate the physical devices
on the system that support Vulkan.

use the [`enumerate_physical_devices`](ScopedInstance::enumerate_physical_devices)
method on a scoped Instance.

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html>
 */

use crate::array_storage::ArrayStorage;
use crate::dispatchable_handles::physical_device::concrete_type::Config;
use crate::dispatchable_handles::physical_device::PhysicalDevices;
use crate::error::Error;
use crate::type_conversions::SafeTransmute;

use crate::dispatchable_handles::instance::Instance;

use super::concrete_type::InstanceConfig;
use super::concrete_type::ScopedInstance;

use vk_safe_sys as vk;

use vk::has_command::EnumeratePhysicalDevices;

impl<S: Instance, C: InstanceConfig> ScopedInstance<S, C>
where
    C::Context: EnumeratePhysicalDevices,
{
    /// enumerate physical devices on the system
    ///
    /// provide an [`ArrayStorage`] implementor to store the PhysicalDevices.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # fn tst<C: vk::instance::VERSION_1_0>(instance: impl vk::Instance<Context = C>) {
    /// let physical_devices = instance.enumerate_physical_devices(Vec::new());
    /// # }
    /// ```
    pub fn enumerate_physical_devices<A: ArrayStorage<vk::PhysicalDevice>>(
        &self,
        mut storage: A,
    ) -> Result<PhysicalDevices<Config<S>, A>, Error> {
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

        // although this SafeTransmute impl seems pointless
        // it is needed for enumerator_code2! to work in general, since most cases involve non trivial transmutes
        unsafe impl SafeTransmute<vk::PhysicalDevice> for vk::PhysicalDevice {}

        let handles = enumerator_code2!(self.context.EnumeratePhysicalDevices().get_fptr(); (self.handle) -> storage)?;
        Ok(PhysicalDevices::new(handles, Config::new(self.scope_ref())))
    }
}
