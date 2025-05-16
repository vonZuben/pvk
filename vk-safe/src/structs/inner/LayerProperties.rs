use std::fmt;

struct_wrapper!(
/// properties of an instance or device layer
///
/// provides the name, description, spec version, and implementation version.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkExtensionProperties.html>
LayerProperties<S,>);

impl<S> LayerProperties<S> {
    get_str!(
        /// Returns the name of the layer
        layer_name);

    get_str!(
        /// Returns the description of th layer
        description);

    pretty_version!(
        /// Returns the Vulkan version the layer was written to
        spec_version);

    /// Return the layers implementation version
    ///
    /// It is an integer, increasing with backward compatible changes.
    pub fn implementation_version(&self) -> u32 {
        self.inner.implementation_version
    }
}

impl<S> std::fmt::Debug for LayerProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayerProperties")
            .field("Name", &self.layer_name())
            .field("Spec Version", &self.spec_version())
            .field("Implementation version", &self.inner.implementation_version)
            .field("Description", &self.description())
            .finish()
    }
}
