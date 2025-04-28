use crate::handles::DispatchableHandle;
use crate::struct_extension::{make_extended, Extended, Pnext};
use crate::type_conversions::ConvertWrapper;
use crate::vk::PhysicalDeviceProperties2;

use vk_safe_sys as vk;

// use vk::has_command::GetPhysicalDeviceProperties2;

pub trait GetPhysicalDeviceProperties2: DispatchableHandle<RawHandle = vk::PhysicalDevice> {
    /// Query the properties of the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_1>>
    /// #   (physical_device: P) {
    /// let properties = physical_device.get_physical_device_properties2(());
    /// # }
    /// ```
    ///
    /// Vulkan docs:
    /// <https://registry.khronos.org/VulkanSC/specs/1.0-extensions/man/html/vkGetPhysicalDeviceProperties2.html>
    fn get_physical_device_properties2<Pn: Pnext<vk::PhysicalDeviceProperties2, Self>, X>(
        &self,
        _p_next: Pn,
    ) -> Extended<PhysicalDeviceProperties2<Self>, Pn::Pnext<Self>>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceProperties2<X>,
    {
        use vk::has_command::GetPhysicalDeviceProperties2;

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

            // init_pnext!
        }

        init_pnext!(
            extension: Pn;
            properties: PhysicalDeviceProperties2<Self>
        );

        unsafe {
            self.commands().GetPhysicalDeviceProperties2().get_fptr()(
                self.raw_handle(),
                properties.as_mut_ptr().to_c(),
            );

            make_extended(properties.assume_init(), extension.assume_init())
        }
    }
}
