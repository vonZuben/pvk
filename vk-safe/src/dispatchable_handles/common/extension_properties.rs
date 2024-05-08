use std::fmt;

simple_struct_wrapper_scoped!(ExtensionProperties);

impl<S> ExtensionProperties<S> {
    get_str!(extension_name);
}

impl<S> std::fmt::Debug for ExtensionProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionProperties")
            .field("Name", &self.extension_name())
            .field("Version", &self.inner.spec_version)
            .finish()
    }
}
