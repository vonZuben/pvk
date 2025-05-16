use crate::spirv::SpirvBinary;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

struct_wrapper!(
/// Info for creating a [`ShaderModule`](crate::vk::ShaderModule)
ShaderModuleCreateInfo<'a,>
);

impl<'a> ShaderModuleCreateInfo<'a> {
    pub fn from_spirv_binary(code: &'a SpirvBinary) -> Self {
        check_vuids::check_vuids!(ShaderModuleCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_codeSize_08735: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, codeSize must be a multiple of 4"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_08736: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, pCode must point to valid SPIR-V code, formatted"
            "and packed as described by the Khronos SPIR-V Specification"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_08737: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, pCode must adhere to the validation rules described"
            "by the Validation Rules within a Module section of the SPIR-V Environment appendix"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_08738: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, pCode must declare the Shader capability for"
            "SPIR-V code"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_08739: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, pCode must not declare any capability that is"
            "not supported by the API, as described by the Capabilities section of the SPIR-V Environment"
            "appendix"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_08740: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, and pCode declares any of the capabilities listed"
            "in the SPIR-V Environment appendix, one of the corresponding requirements must be"
            "satisfied"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_08741: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, pCode must not declare any SPIR-V extension"
            "that is not supported by the API, as described by the Extension section of the SPIR-V"
            "Environment appendix"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_08742: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to SPIR-V code, and pCode declares any of the SPIR-V extensions"
            "listed in the SPIR-V Environment appendix, one of the corresponding requirements must"
            "be satisfied"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_07912: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the VK_NV_glsl_shader extension is not enabled, pCode must be a pointer to SPIR-V"
            "code"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_01379: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pCode is a pointer to GLSL code, it must be valid GLSL code written to the GL_KHR_vulkan_glsl"
            "GLSL extension specification"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_codeSize_01085: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "codeSize must be greater than 0"
            }

            // promised by [SpirvBinary]
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_flags_zerobitmask: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be 0"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkShaderModuleCreateInfo_pCode_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pCode must be a valid pointer to an array of \\(\\textrm{codeSize} \\over 4\\) uint32_t"
            "values"
            }

            // promised by [SpirvBinary]
        }

        unsafe {
            Self::from_c(vk::ShaderModuleCreateInfo {
                s_type: vk::structure_type::SHADER_MODULE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::ShaderModuleCreateFlags::empty(),
                code_size: code.code_size(),
                p_code: code.code_ptr(),
            })
        }
    }
}
