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
impl<S, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn get_physical_device_memory_properties<P>(&self) -> PhysicalDeviceMemoryProperties<S>
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

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceMemoryProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceMemoryProperties_pMemoryProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "pMemoryProperties must be a valid pointer to a VkPhysicalDeviceMemoryProperties structure"
        }

        // MaybeUninit
    }
};

simple_struct_wrapper_scoped!(PhysicalDeviceMemoryProperties);

simple_struct_wrapper_scoped!(MemoryType impl Debug, Deref, Clone, Copy);

impl<S> MemoryType<S> {
    // helper method since the only way to get the property_flags normally
    // is through Deref trait which is not possible const context
    pub(crate) const fn property_flags(&self) -> vk::MemoryPropertyFlags {
        self.inner.property_flags
    }
}

simple_struct_wrapper_scoped!(MemoryHeap impl Debug, Deref);

impl<S> PhysicalDeviceMemoryProperties<S> {
    pub fn memory_types(&self) -> &[MemoryType<S>] {
        assert!(
            self.inner.memory_type_count < vk::MAX_MEMORY_TYPES as _,
            "error: vulkan implementation reporting invalid memory_type_count"
        );
        (&self.inner.memory_types[..self.inner.memory_type_count as _]).safe_transmute_slice()
    }

    pub fn memory_heaps(&self) -> &[MemoryHeap<S>] {
        assert!(
            self.inner.memory_heap_count < vk::MAX_MEMORY_HEAPS as _,
            "error: vulkan implementation reporting invalid memory_heap_count"
        );
        (&self.inner.memory_heaps[..self.inner.memory_heap_count as _]).safe_transmute_slice()
    }

    pub fn choose_type<'a>(&'a self, index: u32) -> MemoryTypeChoice<S> {
        let memory_types = self.memory_types();
        assert!((index as usize) < memory_types.len());
        MemoryTypeChoice {
            ty: self.memory_types()[index as usize],
            index,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MemoryTypeChoice<S> {
    pub(crate) ty: MemoryType<S>,
    pub(crate) index: u32,
}

impl<S> fmt::Debug for PhysicalDeviceMemoryProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceMemoryProperties")
            .field("memory_types", &self.memory_types())
            .field("memory_heaps", &self.memory_heaps())
            .finish()
    }
}
