use super::command_impl_prelude::*;

pub trait EnumerateInstanceLayerProperties {
    fn enumerate_instance_layer_properties<S: EnumeratorStorage<structs::LayerProperties>>(
        &self,
        storage: S,
    ) -> Result<S::InitStorage, vk::Result>;
}

impl_safe_entry_interface! {
EnumerateInstanceLayerProperties {
    enumerator_code!(enumerate_instance_layer_properties() -> structs::LayerProperties);
}}
