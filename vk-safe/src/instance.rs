
pub struct Instance<V: Version, E> {
    feature_commands: V,
    extension_commands: E,
}