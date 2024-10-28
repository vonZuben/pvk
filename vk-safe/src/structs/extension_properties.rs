use std::fmt;

struct_wrapper!(
/// properties of an instance or device extension
///
/// provides the name and version of the extension.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkExtensionProperties.html>
ExtensionProperties<S,>);

impl<S> ExtensionProperties<S> {
    get_str!(
    /// Returns the name of the extension as a &str
    extension_name);

    /// Returns the version of the extension
    ///
    /// It is an integer, incremented with backward compatible changes.
    pub fn version(&self) -> u32 {
        self.inner.spec_version
    }
}

impl<S> std::fmt::Debug for ExtensionProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionProperties")
            .field("Name", &self.extension_name())
            .field("Version", &self.inner.spec_version)
            .finish()
    }
}
