struct_wrapper!(
/// Structure describing external memory host pointer limits that can be supported by an implementation
PhysicalDeviceExternalMemoryHostPropertiesEXT<S,>
impl Deref, Copy, Clone
);

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
