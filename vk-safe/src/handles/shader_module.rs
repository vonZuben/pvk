use super::device::Device;
use super::{Handle, ThreadSafeHandle};

use vk::has_command::DestroyShaderModule;
use vk_safe_sys as vk;

pub trait ShaderModule: Handle<RawHandle = vk::ShaderModule> + ThreadSafeHandle {
    type Device: Device;
}

/// [`ShaderModule`] implementor
struct _ShaderModule<'a, D: Device<Commands: DestroyShaderModule>> {
    handle: vk::ShaderModule,
    device: &'a D,
}

pub(crate) fn make_shader_module<'a, D: Device<Commands: DestroyShaderModule>>(
    device: &'a D,
    handle: vk::ShaderModule,
) -> impl ShaderModule<Device = D> + use<'a, D> {
    _ShaderModule { handle, device }
}

impl<'a, D: Device<Commands: DestroyShaderModule>> ShaderModule for _ShaderModule<'a, D> {
    type Device = D;
}

unsafe impl<'a, D: Device<Commands: DestroyShaderModule>> Send for _ShaderModule<'a, D> {}
unsafe impl<'a, D: Device<Commands: DestroyShaderModule>> Sync for _ShaderModule<'a, D> {}
impl<'a, D: Device<Commands: DestroyShaderModule>> ThreadSafeHandle for _ShaderModule<'a, D> {}

impl<'a, D: Device<Commands: DestroyShaderModule>> Handle for _ShaderModule<'a, D> {
    type RawHandle = vk::ShaderModule;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<'a, D: Device<Commands: DestroyShaderModule>> std::fmt::Debug for _ShaderModule<'a, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderModule")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<'a, D: Device<Commands: DestroyShaderModule>> Drop for _ShaderModule<'a, D> {
    fn drop(&mut self) {
        check_vuids::check_vuids!(DestroyShaderModule);

        #[allow(unused_labels)]
        'VUID_vkDestroyShaderModule_shaderModule_01092: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkAllocationCallbacks were provided when shaderModule was created, a compatible"
            "set of callbacks must be provided here"
            }

            // always null for now
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyShaderModule_shaderModule_01093: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If no VkAllocationCallbacks were provided when shaderModule was created, pAllocator"
            "must be NULL"
            }

            // always null for now
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyShaderModule_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "device must be a valid VkDevice handle"
            }

            // ensured by Device creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyShaderModule_shaderModule_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If shaderModule is not VK_NULL_HANDLE, shaderModule must be a valid VkShaderModule"
            "handle"
            }

            // ensured by ShaderModule creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyShaderModule_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // always null for now
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyShaderModule_shaderModule_parent: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If shaderModule is a valid handle, it must have been created, allocated, or retrieved"
            "from device"
            }

            // ensured by ShaderModule creation
        }

        unsafe {
            self.device.commands().DestroyShaderModule().get_fptr()(
                self.device.raw_handle(),
                self.handle,
                std::ptr::null(),
            )
        }
    }
}
