use krs_quote::{ToTokens, krs_quote_with};
use crate::utils::VkTyName;
use crate::vuid_visitor::VuidPair;

pub struct VuidCollection<'a> {
    target: VkTyName,
    vuid_pair: Vec<VuidPair<'a>>,
}

impl ToTokens for VuidCollection<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let target = self.target;
        let _pairs = &self.vuid_pair;

        krs_quote_with!(tokens <-
            pub trait {@target} {

            }
        );
    }
}