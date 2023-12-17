use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::physical_device::PhysicalDevices;

use crate::instance::Instance;

use super::InstanceConfig;
use super::ScopedInstanceType;

use vk_safe_sys as vk;

use vk::has_command::EnumeratePhysicalDevices;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html
*/
impl<S: Instance, C: InstanceConfig> ScopedInstanceType<S, C> {
    pub fn enumerate_physical_devices<P, A: ArrayStorage<vk::PhysicalDevice>>(
        &self,
        mut storage: A,
    ) -> Result<PhysicalDevices<S, A>, Error>
    where
        C::Commands: EnumeratePhysicalDevices<P>,
    {
        check_vuids::check_vuids!(EnumeratePhysicalDevices);

        #[allow(unused_labels)]
        'VUID_vkEnumeratePhysicalDevices_instance_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "instance must be a valid VkInstance handle"
            }

            // always valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkEnumeratePhysicalDevices_pPhysicalDeviceCount_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pPhysicalDeviceCount must be a valid pointer to a uint32_t value"
            }

            // enumerator_code2!
        }

        #[allow(unused_labels)]
        'VUID_vkEnumeratePhysicalDevices_pPhysicalDevices_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the value referenced by pPhysicalDeviceCount is not 0, and pPhysicalDevices is"
            "not NULL, pPhysicalDevices must be a valid pointer to an array of pPhysicalDeviceCount"
            "VkPhysicalDevice handles"
            }

            //enumerator_code2!
        }

        let handles = enumerator_code2!(self.commands.EnumeratePhysicalDevices().get_fptr(); (self.handle) -> storage)?;
        Ok(PhysicalDevices::new(handles, self.to_scope()))
    }
}
