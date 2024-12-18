//! Generate normalized structure type names
//!
//! Normalized structure type names refer to the
//! values of `StructureType` so they can be consistently referred
//! to by generated `BaseStructure` trait implementations.

use super::*;

use crate::utils::{case, StrAsCode};

pub struct Delegate;

impl<'a, I: Variants<'a>> ToTokensDelegate<I> for Delegate {
    fn delegate_to_tokens(params: &Properties<I>, tokens: &mut TokenStream) {
        let mut last = None;
        let variant_names = params.variants.clone().filter_map(move |v| {
            if v.is_alias() {
                return None;
            }

            // The values VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO and VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO
            // are reserved for internal use by the loader, and do not have corresponding Vulkan structures in this Specification.
            if matches!(
                v.name().as_str(),
                "LOADER_INSTANCE_CREATE_INFO" | "LOADER_DEVICE_CREATE_INFO"
            ) {
                return None;
            }

            let norm = case::normalize(v.name());

            if Some(&norm) == last.as_ref() {
                return None;
            }

            last = Some(norm.clone());
            Some((v.name(), norm))
        });

        let normalized_struct_names = variant_names.clone().map(|t| t.1.as_code());

        let real_names = variant_names.clone().map(|t| t.0);

        krs_quote_with!(tokens <-
            {@*
                #[allow(non_upper_case_globals)]
                pub(crate) const {@normalized_struct_names}: StructureType = StructureType::{@real_names};
            }
        );
    }
}
