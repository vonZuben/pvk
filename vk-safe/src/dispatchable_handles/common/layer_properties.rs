use std::fmt;

simple_struct_wrapper_scoped!(LayerProperties);

impl<S> LayerProperties<S> {
    get_str!(layer_name);
    get_str!(description);
    pretty_version!(spec_version);
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
