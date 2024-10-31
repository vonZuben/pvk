use super::Device;

use crate::error::Error;
use crate::handles::shader_module::{_ShaderModule, make_shader_module};
use crate::type_conversions::ConvertWrapper;
use crate::vk::ShaderModuleCreateInfo;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use vk::has_command::{CreateShaderModule, DestroyShaderModule};

pub(crate) fn create_shader_module<
    'a,
    D: Device<Commands: CreateShaderModule + DestroyShaderModule>,
>(
    device: &'a D,
    info: &ShaderModuleCreateInfo,
) -> Result<_ShaderModule<'a, D>, Error> {
    check_vuids::check_vuids!(CreateShaderModule);

    #[allow(unused_labels)]
    'VUID_vkCreateShaderModule_pCreateInfo_06904: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If pCreateInfo is not NULL, pCreateInfo-&gt;pNext must be NULL or a pointer to a VkShaderModuleValidationCacheCreateInfoEXT"
        "structure"
        }

        // ensured by ShaderModuleCreateInfo creation
    }

    #[allow(unused_labels)]
    'VUID_vkCreateShaderModule_device_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "device must be a valid VkDevice handle"
        }

        // ensured by Device creation
    }

    #[allow(unused_labels)]
    'VUID_vkCreateShaderModule_pCreateInfo_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pCreateInfo must be a valid pointer to a valid VkShaderModuleCreateInfo structure"
        }

        // ensured by ShaderModuleCreateInfo creation
    }

    #[allow(unused_labels)]
    'VUID_vkCreateShaderModule_pAllocator_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
        "structure"
        }

        // TODO always null for now
    }

    #[allow(unused_labels)]
    'VUID_vkCreateShaderModule_pShaderModule_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pShaderModule must be a valid pointer to a VkShaderModule handle"
        }

        // Maybeuninit
    }

    let mut handle = MaybeUninit::uninit();
    unsafe {
        let res = device.commands().CreateShaderModule().get_fptr()(
            device.raw_handle(),
            info.to_c(),
            std::ptr::null(),
            handle.as_mut_ptr(),
        );
        check_raw_err!(res);
        Ok(make_shader_module(device, handle.assume_init()))
    }
}
