use crate::vk::PhysicalDevice;
use crate::vk::PhysicalDeviceProperties2;

use crate::struct_extension::{make_extended, Extended, Pnext};
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceProperties2;

pub(crate) fn get_physical_device_properties2<
    Pd: PhysicalDevice<Commands: GetPhysicalDeviceProperties2<X>>,
    Pn: Pnext<vk::PhysicalDeviceProperties2>,
    X,
>(
    physical_device: &Pd,
    _p_next: Pn,
) -> Extended<PhysicalDeviceProperties2<Pd>, Pn::Pnext<Pd>> {
    check_vuids::check_vuids!(GetPhysicalDeviceProperties2);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceProperties2_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // ensured by PhysicalDevice creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceProperties2_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pProperties must be a valid pointer to a VkPhysicalDeviceProperties2 structure"
        }

        // PartialInitMut
    }

    init_pnext!(
        extension: Pn;
        properties: PhysicalDeviceProperties2<Pd>
    );

    unsafe {
        physical_device
            .commands()
            .GetPhysicalDeviceProperties2()
            .get_fptr()(physical_device.raw_handle(), properties.as_mut_ptr().to_c());

        make_extended(
            ConvertWrapper::from_c(properties.assume_init()),
            extension.assume_init(),
        )
    }
}
