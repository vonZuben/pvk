use super::*;
use crate::instance::InstanceConfig;
use crate::safe_interface::type_conversions::TransmuteArray;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;
use std::mem::MaybeUninit;

use vk_safe_sys::validation::GetPhysicalDeviceMemoryProperties::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html
*/
impl<'scope, C: InstanceConfig> ScopedPhysicalDevice<'scope, '_, C>
where
    C::Commands: vk::GetCommand<vk::GetPhysicalDeviceMemoryProperties>,
{
    pub fn get_physical_device_memory_properties(&self) -> PhysicalDeviceMemoryProperties<'scope> {
        validate(Validation);
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance.commands.get().get_fptr()(
                self.handle,
                properties.as_mut_ptr()
            );
            PhysicalDeviceMemoryProperties::new(properties.assume_init())
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

pub struct PhysicalDeviceMemoryProperties<'scope> {
    inner: vk::PhysicalDeviceMemoryProperties,
    _scope: ScopeId<'scope>,
}

simple_struct_wrapper_scoped!(MemoryType);

impl std::fmt::Debug for MemoryType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

simple_struct_wrapper_scoped!(MemoryHeap);

impl std::fmt::Debug for MemoryHeap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'scope> PhysicalDeviceMemoryProperties<'scope> {
    fn new(inner: vk::PhysicalDeviceMemoryProperties) -> Self {
        Self { inner, _scope: Default::default() }
    }
    // TODO, I think the MemoryType should als be scoped
    pub fn memory_types(&self) -> &[MemoryType<'scope>] {
        assert!(self.inner.memory_type_count < vk::MAX_MEMORY_TYPES as _, "error: vulkan implementation reporting invalid memory_type_count");
        (&self.inner.memory_types[..self.inner.memory_type_count as _]).safe_transmute()
    }
    // TODO, I think the MemoryHeap should als be scoped
    pub fn memory_heaps(&self) -> &[MemoryHeap<'scope>] {
        assert!(self.inner.memory_heap_count < vk::MAX_MEMORY_HEAPS as _, "error: vulkan implementation reporting invalid memory_heap_count");
        (&self.inner.memory_heaps[..self.inner.memory_heap_count as _]).safe_transmute()
    }
}

impl fmt::Debug for PhysicalDeviceMemoryProperties<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceMemoryProperties")
            .field("memory_types", &self.memory_types())
            .field("memory_heaps", &self.memory_heaps())
            .finish()
    }
}