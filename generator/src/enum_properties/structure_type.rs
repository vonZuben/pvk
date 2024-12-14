//! Generate normalized structure type names
//!
//! Normalized structure type names refer to the
//! values of `StructureType` so they can be consistently referred
//! to by generated `BaseStructure` trait implementations.

use super::*;

use crate::utils::{case, StrAsCode};

pub struct Delegate;

impl<I: Iterator<Item = VkTyName> + Clone> ToTokensDelegate<I> for Delegate {
    fn delegate_to_tokens(params: &Properties<I>, tokens: &mut TokenStream) {
        let mut last = None;
        let variants = params.variants.clone().filter_map(move |v| {
            let norm = case::normalize(&v);
            if Some(&norm) == last.as_ref() {
                None
            } else {
                last = Some(norm.clone());
                Some((v, norm))
            }
        });

        let normalized_struct_names = variants.clone().map(|t| t.1.as_code());

        let real_names = variants.clone().map(|t| t.0);

        krs_quote_with!(tokens <-
            {@* const {@normalized_struct_names}: StructureType = StructureType::{@real_names}; }
        );
    }
}
