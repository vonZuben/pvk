use super::*;
use crate::instance::Instance;
use crate::type_conversions::TransmuteSlice;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceMemoryProperties;

use std::fmt;
use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html
*/
impl<'scope, I: Instance> ScopedPhysicalDeviceType<'scope, I> {
    pub fn get_physical_device_memory_properties<P>(&self) -> PhysicalDeviceMemoryProperties<'scope>
    where
        I::Commands: GetPhysicalDeviceMemoryProperties<P>,
    {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance
                .commands
                .GetPhysicalDeviceMemoryProperties()
                .get_fptr()(self.handle, properties.as_mut_ptr());
            PhysicalDeviceMemoryProperties::new(properties.assume_init())
        }
    }
}

const _VUID: () = {
    check_vuids::check_vuids!(GetPhysicalDeviceMemoryProperties);
    // check_vuid_defs2!( GetPhysicalDeviceMemoryProperties
    //     pub const VUID_vkGetPhysicalDeviceMemoryProperties_physicalDevice_parameter:
    //         &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
    //     // PhysicalDevice
    //     pub const VUID_vkGetPhysicalDeviceMemoryProperties_pMemoryProperties_parameter : & 'static [ u8 ] = "pMemoryProperties must be a valid pointer to a VkPhysicalDeviceMemoryProperties structure" . as_bytes ( ) ;
    //     // MaybeUninit
    // )
};

simple_struct_wrapper_scoped!(PhysicalDeviceMemoryProperties);

simple_struct_wrapper_scoped!(MemoryType impl Debug, Deref, Clone, Copy);

impl MemoryType<'_> {
    // helper method since the only way to get the property_flags normally
    // is through Deref trait which is not possible const context
    pub(crate) const fn property_flags(&self) -> vk::MemoryPropertyFlags {
        self.inner.property_flags
    }
}

simple_struct_wrapper_scoped!(MemoryHeap impl Debug, Deref);

impl<'scope> PhysicalDeviceMemoryProperties<'scope> {
    pub fn memory_types(&self) -> &[MemoryType<'scope>] {
        assert!(
            self.inner.memory_type_count < vk::MAX_MEMORY_TYPES as _,
            "error: vulkan implementation reporting invalid memory_type_count"
        );
        (&self.inner.memory_types[..self.inner.memory_type_count as _]).safe_transmute_slice()
    }

    pub fn memory_heaps(&self) -> &[MemoryHeap<'scope>] {
        assert!(
            self.inner.memory_heap_count < vk::MAX_MEMORY_HEAPS as _,
            "error: vulkan implementation reporting invalid memory_heap_count"
        );
        (&self.inner.memory_heaps[..self.inner.memory_heap_count as _]).safe_transmute_slice()
    }

    pub fn choose_type<'a>(&'a self, index: u32) -> MemoryTypeChoice<'scope> {
        let memory_types = self.memory_types();
        assert!((index as usize) < memory_types.len());
        MemoryTypeChoice {
            ty: self.memory_types()[index as usize],
            index,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MemoryTypeChoice<'scope> {
    pub(crate) ty: MemoryType<'scope>,
    pub(crate) index: u32,
}

impl fmt::Debug for PhysicalDeviceMemoryProperties<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceMemoryProperties")
            .field("memory_types", &self.memory_types())
            .field("memory_heaps", &self.memory_heaps())
            .finish()
    }
}
