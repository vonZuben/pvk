struct_wrapper!(
/// Structure describing external memory host pointer limits that can be supported by an implementation
PhysicalDeviceExternalMemoryHostPropertiesEXT<S,>
impl Deref, Copy, Clone
);

const _: () = {
    check_vuids::check_vuids!(PhysicalDeviceExternalMemoryHostPropertiesEXT);

    #[allow(unused_labels)]
    'VUID_VkPhysicalDeviceExternalMemoryHostPropertiesEXT_sType_sType: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "sType must be VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT"
        }

        // see struct_extensions.rs
    }
};

impl<S> std::fmt::Debug for PhysicalDeviceExternalMemoryHostPropertiesEXT<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalDeviceExternalMemoryHostPropertiesEXT")
            .field(
                "min_imported_host_pointer_alignment",
                &self.inner.min_imported_host_pointer_alignment,
            )
            .finish()
    }
}
