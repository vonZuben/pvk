use super::*;
use crate::instance_type::Instance;
use crate::type_conversions::TransmuteSlice;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceMemoryProperties;

use crate::flags::Flags;

use std::fmt;
use std::mem::MaybeUninit;

use std::marker::PhantomData;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html
*/
impl<S, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn get_physical_device_memory_properties(&self) -> PhysicalDeviceMemoryProperties<S>
    where
        I::Commands: GetPhysicalDeviceMemoryProperties,
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
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceMemoryProperties_pMemoryProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pMemoryProperties must be a valid pointer to a VkPhysicalDeviceMemoryProperties structure"
        }

        // MaybeUninit
    }
};

simple_struct_wrapper_scoped!(PhysicalDeviceMemoryProperties);

simple_struct_wrapper_scoped!(MemoryType impl Debug, Deref, Clone, Copy);

simple_struct_wrapper_scoped!(MemoryHeap impl Debug, Deref, Clone, Copy);

unit_error!(pub InvalidMemoryType);

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

    /// choose the given index as a memory type for other operations (e.g. to allocate)
    /// Returns Ok with the chosen type if it has the correct MemoryPropertyFlags and MemoryHeapFlags
    /// otherwise return an error indicating that the memory type is not appropriate
    pub fn choose_type<
        'a,
        P: Flags<Type = vk::MemoryPropertyFlags>,
        H: Flags<Type = vk::MemoryHeapFlags>,
    >(
        &'a self,
        index: u32,
        _property_flags: P,
        _heap_flags: H,
    ) -> Result<MemoryTypeChoice<S, P, H>, InvalidMemoryType> {
        let memory_types = self.memory_types();

        if (index as usize) < memory_types.len() {
            let ty = self.memory_types()[index as usize];
            let heap = self.memory_heaps()[ty.heap_index as usize];

            if ty.property_flags.contains(P::FLAGS) && !heap.flags.contains(H::NOT_FLAGS) {
                return Ok(MemoryTypeChoice {
                    scope: PhantomData,
                    index,
                    property_flags: PhantomData,
                    heap_flags: PhantomData,
                });
            }
        }

        Err(InvalidMemoryType)
    }

    /// find the first memory type that satisfies the given MemoryPropertyFlags and MemoryHeapFlags
    ///
    /// This is a convenience function. If employing more advanced memory management, it will be better to
    /// consider all available memory types more carefully.
    pub fn find_ty<
        'a,
        P: Flags<Type = vk::MemoryPropertyFlags>,
        H: Flags<Type = vk::MemoryHeapFlags>,
    >(
        &'a self,
        _property_flags: P,
        _heap_flags: H,
    ) -> Option<MemoryTypeChoice<S, P, H>> {
        for (index, ty) in self.memory_types().iter().enumerate() {
            let heap = self.memory_heaps()[ty.heap_index as usize];

            if ty.property_flags.contains(P::FLAGS) && !heap.flags.contains(H::NOT_FLAGS) {
                return Some(MemoryTypeChoice {
                    scope: PhantomData,
                    index: index as u32, // should be a safe as cast since we assume the number of memory types to enumerate is valid
                    property_flags: PhantomData,
                    heap_flags: PhantomData,
                });
            }
        }

        None
    }
}

#[derive(Clone, Copy)]
pub struct MemoryTypeChoice<S, P, H> {
    scope: PhantomData<S>,
    // pub(crate) ty: MemoryType<S>,
    pub(crate) index: u32,
    property_flags: PhantomData<P>,
    heap_flags: PhantomData<H>,
}

impl<S> fmt::Debug for PhysicalDeviceMemoryProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceMemoryProperties")
            .field("memory_types", &self.memory_types())
            .field("memory_heaps", &self.memory_heaps())
            .finish()
    }
}
