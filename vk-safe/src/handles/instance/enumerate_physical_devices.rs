use super::Instance;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::handles::physical_device::{make_physical_devices, PhysicalDevices};
use crate::scope::Captures;
use crate::type_conversions::SafeTransmute;

use vk_safe_sys as vk;

use vk::has_command::EnumeratePhysicalDevices;

pub(crate) fn enumerate_physical_devices<
    'a,
    I: Instance<Commands: EnumeratePhysicalDevices>,
    A: ArrayStorage<vk::PhysicalDevice>,
>(
    instance: &'a I,
    mut storage: A,
) -> Result<impl PhysicalDevices<I> + Captures<&'a I>, Error> {
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

    let handles = enumerator_code2!(instance.commands().EnumeratePhysicalDevices().get_fptr(); (instance.raw_handle()) -> storage)?;
    Ok(make_physical_devices(instance, handles))
}
