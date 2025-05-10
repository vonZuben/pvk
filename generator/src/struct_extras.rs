mod physical_device_features;

use crate::types::Struct2;

use krs_quote::ToTokens;

#[derive(Default)]
/// Generates bespoke code for unique structs
pub struct StructExtras {
    physical_device_features: physical_device_features::PhysicalDeviceFeatures,
}

impl StructExtras {
    pub fn check_bespoke(&mut self, s: &Struct2) {
        self.physical_device_features.delegate_check_bespoke(s);
    }
}

impl ToTokens for StructExtras {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        self.physical_device_features.delegate_to_tokens(tokens);
    }
}

trait ExtrasDelegate: Sized {
    fn delegate_to_tokens(&self, tokens: &mut krs_quote::TokenStream);
    fn delegate_check_bespoke(&mut self, s: &Struct2);
}
