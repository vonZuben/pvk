use super::Instance;

use crate::enumerator::Enumerator;
use crate::handles::physical_device::PhysicalDeviceHandle;
use crate::scope::Captures;

use vk_safe_sys as vk;

use vk::has_command::EnumeratePhysicalDevices;

pub(crate) fn enumerate_physical_devices<
    'a,
    I: Instance<Commands: EnumeratePhysicalDevices<X>>,
    X,
>(
    instance: &'a I,
) -> impl Enumerator<PhysicalDeviceHandle<I>> + Captures<&'a I> {
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

    make_enumerator!(instance.commands().EnumeratePhysicalDevices().get_fptr(); (instance.raw_handle()))
}
