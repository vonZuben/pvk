use std::mem::MaybeUninit;

use crate::error::Error;
use crate::handles::device::{make_device, Device};
use crate::handles::DispatchableHandle;
use crate::scope::{Captures, HasScope, Tag};
use crate::structs::DeviceCreateInfo;
use crate::type_conversions::ConvertWrapper;

pub(crate) use private::VersionCheck;

use vk_safe_sys as vk;

use vk::context::{Context, InstanceDependencies, LoadCommands};
use vk::has_command::DestroyDevice;
use vk::Version;

mod private {
    use vk_safe_sys::Version;

    pub trait VersionCheck<I> {
        const VALID: ();
    }

    impl<I: Version, D: Version> VersionCheck<I> for D {
        const VALID: () = {
            if D::VERSION.raw() > I::VERSION.raw() {
                panic!("version of Instance must be >= version of Device")
            }
        };
    }
}

pub trait CreateDevice: DispatchableHandle<RawHandle = vk::PhysicalDevice> {
    fn create_device<'t, Ctx, Z: HasScope<Self>, Create, Destroy>(
        &self,
        create_info: &DeviceCreateInfo<Ctx, Z>,
        tag: Tag<'t>,
    ) -> Result<
        impl Device<Commands = Ctx::Commands, PhysicalDevice = Self, QueueConfig = Z>
            + Captures<(Tag<'t>, Self, Ctx, Z)>
            + use<'t, Self, Ctx, Z, Create, Destroy>,
        Error,
    >
    where
        Self::Commands: vk::has_command::CreateDevice<Create>,
        Ctx: Context + Send + Sync,
        Ctx::Commands: DestroyDevice<Destroy>
            + LoadCommands
            + Version
            + VersionCheck<Self::Commands>
            + InstanceDependencies<Self::Commands>
            + Send
            + Sync,
    {
        use vk::has_command::CreateDevice;

        // check version requirement
        let _ = Ctx::Commands::VALID;

        check_vuids::check_vuids!(CreateDevice);

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_ppEnabledExtensionNames_01387: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "All required device extensions for each extension in the VkDeviceCreateInfo::ppEnabledExtensionNames"
            "list must also be present in that list"
            }

            // This is ensured by the context creation macros and InstanceDependencies trait
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // ensured by PhysicalDevice creation
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_pCreateInfo_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pCreateInfo must be a valid pointer to a valid VkDeviceCreateInfo structure"
            }

            // ensured by DeviceCreateInfo creation
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // AllocationCallbacks not supported
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_pDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pDevice must be a valid pointer to a VkDevice handle"
            }

            // MaybeUninit
        }

        let mut handle = MaybeUninit::uninit();

        // // *********************************************
        // // *********Fix with extension support**********
        // // **VUID_VkDeviceCreateInfo_pProperties_04451**
        // // *********************************************
        // for e in self
        //     .commands()
        //     .enumerate_device_extension_properties(None)
        //     .auto_get_enumerate()?
        // {
        //     if e.extension_name() == "VK_KHR_portability_subset" {
        //         panic!("Physical device with VK_KHR_portability_subset is not supported")
        //     }
        // }
        // // *********************************************

        let device;
        unsafe {
            let res = self.commands().CreateDevice().get_fptr()(
                self.raw_handle(),
                create_info.to_c(),
                std::ptr::null(),
                handle.as_mut_ptr(),
            );
            check_raw_err!(res);
            device = handle.assume_init();
        }
        let loader = |command_name| unsafe { vk::GetDeviceProcAddr(device, command_name) };
        Ok(make_device(device, Ctx::Commands::load(loader)?, tag))
    }
}
