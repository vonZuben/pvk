use std::fmt;

use super::concrete_type::ScopedPhysicalDeviceType;

use crate::dispatchable_handles::instance::Instance;

use crate::type_conversions::TransmuteSlice;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceMemoryProperties;

use crate::flags::Flags;

use std::mem::MaybeUninit;

use std::marker::PhantomData;

impl<S, I: Instance> ScopedPhysicalDeviceType<S, I>
where
    I::Context: GetPhysicalDeviceMemoryProperties,
{
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html>
    pub fn get_physical_device_memory_properties(&self) -> PhysicalDeviceMemoryProperties<S> {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance
                .context
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
    ///
    /// **⚠️ VK_MEMORY_PROPERTY_DEVICE_COHERENT_BIT_AMD is not supported**
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

            MemoryTypeChoice::new(index, &ty, &heap).map_err(|e| match e {
                InternalInvalidMemoryType::DesiredFlagsNotAvailable => InvalidMemoryType,
                InternalInvalidMemoryType::NotSupportedDeviceCoherentBitAMD =>
                panic!("This memory type includes VK_MEMORY_PROPERTY_DEVICE_COHERENT_BIT_AMD which is not currently supported by vk-safe.
                Please select a different type or use `find_ty` to automatically choose an appropriate type if possible"),
            })
        } else {
            Err(InvalidMemoryType)
        }
    }

    /// find the first memory type that satisfies the given MemoryPropertyFlags and MemoryHeapFlags
    ///
    /// This is a convenience function. If employing more advanced memory management, it will be better to
    /// consider all available memory types more carefully.
    ///
    /// **⚠️ VK_MEMORY_PROPERTY_DEVICE_COHERENT_BIT_AMD is not supported**
    pub fn find_ty<
        'a,
        P: Flags<Type = vk::MemoryPropertyFlags>,
        H: Flags<Type = vk::MemoryHeapFlags>,
    >(
        &'a self,
        _property_flags: P,
        _heap_flags: H,
    ) -> Option<MemoryTypeChoice<S, P, H>> {
        if P::INCLUDES.contains(vk::MemoryPropertyFlags::DEVICE_COHERENT_BIT_AMD) {
            panic!("Do not request VK_MEMORY_PROPERTY_DEVICE_COHERENT_BIT_AMD because it is not currently supported by vk-safe.")
        }

        for (index, ty) in self.memory_types().iter().enumerate() {
            let heap = self.memory_heaps()[ty.heap_index as usize];

            // index should be a safe to cast since we assume the number of memory types to enumerate is valid
            match MemoryTypeChoice::new(index as u32, ty, &heap) {
                Ok(choice) => return Some(choice),
                Err(_) => {} // regardless of reason we cannot choose this type
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

/// internal errors when choosing a memory type
enum InternalInvalidMemoryType {
    /// The users desired flag set is not available
    DesiredFlagsNotAvailable,

    /// VK_MEMORY_PROPERTY_DEVICE_COHERENT_BIT_AMD is not
    /// supported because it requires a certain feature to be
    /// enabled which is not currently checked
    NotSupportedDeviceCoherentBitAMD,
}

impl<S, P: Flags<Type = vk::MemoryPropertyFlags>, H: Flags<Type = vk::MemoryHeapFlags>>
    MemoryTypeChoice<S, P, H>
{
    fn new(
        index: u32,
        ty: &MemoryType<S>,
        heap: &MemoryHeap<S>,
    ) -> Result<Self, InternalInvalidMemoryType> {
        if ty
            .property_flags
            .contains(vk::MemoryPropertyFlags::DEVICE_COHERENT_BIT_AMD)
        {
            return Err(InternalInvalidMemoryType::NotSupportedDeviceCoherentBitAMD);
        }

        if ty.property_flags.contains(P::INCLUDES) && heap.flags.excludes(H::EXCLUDES) {
            Ok(Self {
                scope: PhantomData,
                index,
                property_flags: PhantomData,
                heap_flags: PhantomData,
            })
        } else {
            Err(InternalInvalidMemoryType::DesiredFlagsNotAvailable)
        }
    }
}

impl<S> fmt::Debug for PhysicalDeviceMemoryProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceMemoryProperties")
            .field("memory_types", &self.memory_types())
            .field("memory_heaps", &self.memory_heaps())
            .finish()
    }
}
