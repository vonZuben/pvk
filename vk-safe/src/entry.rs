use vk_safe_sys as vk;

fn entry_fn_loader<C: vk::VulkanCommand>() -> Option<C> {
    // Safe because null is valid instance for global/entry commands, and vk::VulkanCommand ensures we provide a proper p_name
    // https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html
    unsafe {
        let fptr = vk::GetInstanceProcAddr(
            vk::Instance {
                handle: std::ptr::null(),
            },
            C::VK_NAME,
        )?;
        Some(C::new(fptr))
    }
}

// The following is imported by each command impl module
mod command_impl_prelude {
    pub use vk_safe_sys as vk;
}

mod create_instance;
mod enumerate_instance_extension_properties;
mod enumerate_instance_layer_properties;
mod enumerate_instance_version;

pub use create_instance::*;
pub use enumerate_instance_extension_properties::*;
pub use enumerate_instance_layer_properties::*;
pub use enumerate_instance_version::*;
