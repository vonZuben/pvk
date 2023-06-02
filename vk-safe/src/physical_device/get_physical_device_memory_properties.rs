use super::*;
use crate::instance::InstanceConfig;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;
use std::mem::MaybeUninit;

use vk_safe_sys::validation::GetPhysicalDeviceMemoryProperties::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C>
where
    C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceMemoryProperties>,
{
    pub fn get_physical_device_memory_properties(&self) -> PhysicalDeviceMemoryProperties {
        validate(Validation);
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance.feature_commands.get().get_fptr()(
                self.handle,
                properties.as_mut_ptr()
            );
            PhysicalDeviceMemoryProperties { inner: properties.assume_init() }
        }
    }
}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkGetPhysicalDeviceMemoryProperties_physicalDevice_parameter: () ={
        // PhysicalDevice
    };

    const VUID_vkGetPhysicalDeviceMemoryProperties_pMemoryProperties_parameter: () = {
        // MaybeUninit
    };
}

check_vuid_defs!(
    pub const VUID_vkGetPhysicalDeviceMemoryProperties_physicalDevice_parameter:
            &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        pub const VUID_vkGetPhysicalDeviceMemoryProperties_pMemoryProperties_parameter : & 'static [ u8 ] = "pMemoryProperties must be a valid pointer to a VkPhysicalDeviceMemoryProperties structure" . as_bytes ( ) ;
);

pub struct PhysicalDeviceMemoryProperties {
    inner: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDeviceMemoryProperties {
    pub fn memory_types(&self) -> &[vk::MemoryType] {
        assert!(self.inner.memory_type_count < vk::MAX_MEMORY_TYPES as _, "error: vulkan implementation reporting invalid memory_type_count");
        &self.inner.memory_types[..self.inner.memory_type_count as _]
    }
    pub fn memory_heaps(&self) -> &[vk::MemoryHeap] {
        assert!(self.inner.memory_heap_count < vk::MAX_MEMORY_HEAPS as _, "error: vulkan implementation reporting invalid memory_heap_count");
        &self.inner.memory_heaps[..self.inner.memory_heap_count as _]
    }
}

impl fmt::Debug for PhysicalDeviceMemoryProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceMemoryProperties")
            .field("memory_types", &self.memory_types())
            .field("memory_heaps", &self.memory_heaps())
            .finish()
    }
}