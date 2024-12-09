use crate::error::Error;
use crate::handles::instance::{make_instance, Instance};
use crate::scope::{Captures, Tag};
use crate::structs::InstanceCreateInfo;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use vk::context::{Context, LoadCommands};
use vk::has_command::DestroyInstance;
use vk::Version;

/// Create an instance
///
/// In order to create an Instance, you first define the Version and Extensions you will use with [`vk::instance_context!`]. You can then create an
/// [`ApplicationInfo`](crate::structs::ApplicationInfo) structure, and subsequently create an [`InstanceCreateInfo`] structure for
/// passing to this function.
///
/// See also
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateInstance.html>
pub fn create_instance<'t, C: Context>(
    create_info: &InstanceCreateInfo<C>,
    tag: Tag<'t>,
) -> Result<impl Instance<Commands = C::Commands> + Captures<Tag<'t>>, Error>
where
    C::Commands: DestroyInstance + Version + LoadCommands,
{
    check_vuids::check_vuids!(CreateInstance);

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_ppEnabledExtensionNames_01388: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "All required extensions for each extension in the VkInstanceCreateInfo::ppEnabledExtensionNames"
        "list must also be present in that list"
        }

        // This is ensured by the context creation macros
    }

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_pCreateInfo_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pCreateInfo must be a valid pointer to a valid VkInstanceCreateInfo structure"
        }

        // rust reference; CreateInfo validated on its own
    }

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_pAllocator_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
        "structure"
        }

        // TODO: not currently supported, always set to NULL
    }

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_pInstance_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pInstance must be a valid pointer to a VkInstance handle"
        }

        // MaybeUninit
    }

    // TODO: return proper error for failing to load the command
    let command = super::entry_fn_loader::<vk::CreateInstance>()
        .unwrap()
        .get_fptr();

    let mut handle = MaybeUninit::uninit();
    let instance;
    unsafe {
        let res = command(&create_info.inner, std::ptr::null(), handle.as_mut_ptr());
        check_raw_err!(res);
        instance = handle.assume_init();
    }
    let loader = |command_name| unsafe { vk::GetInstanceProcAddr(instance, command_name) };
    Ok(make_instance(instance, C::Commands::load(loader)?, tag))
}
