use crate::error::Error;
use crate::handles::shader_module::{make_shader_module, ShaderModule};
use crate::handles::DispatchableHandle;
use crate::type_conversions::ConvertWrapper;
use crate::vk::ShaderModuleCreateInfo;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

pub trait CreateShaderModule: DispatchableHandle<RawHandle = vk::Device> {
    /// Create a ShaderModule
    ///
    /// A [`ShaderModule`] contains shader code and one or more entry points. Shaders are selected from a
    /// shader module by specifying an entry point as part of pipeline creation. The stages of a pipeline
    /// can use shaders that come from different modules. The shader code defining a shader module must be
    /// in the SPIR-V format, as described by the Vulkan Environment for SPIR-V appendix.
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateShaderModule.html>
    fn create_shader_module<'a, X, Y>(
        &'a self,
        info: &ShaderModuleCreateInfo,
    ) -> Result<impl ShaderModule<Device = Self> + use<'a, Self, X, Y>, Error>
    where
        Self::Commands:
            vk::has_command::CreateShaderModule<X> + vk::has_command::DestroyShaderModule<Y>,
    {
        use vk::has_command::CreateShaderModule;

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

            // MaybeUninit
        }

        let mut handle = MaybeUninit::uninit();
        unsafe {
            let res = self.commands().CreateShaderModule().get_fptr()(
                self.raw_handle(),
                info.to_c(),
                std::ptr::null(),
                handle.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(make_shader_module(self, handle.assume_init()))
        }
    }
}
