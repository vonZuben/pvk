use crate::utils::VkTyName;
use krs_quote::{krs_quote_with, to_tokens_closure, ToTokens};

pub(crate) fn dependencies_to_tokens(
    features: impl Iterator<Item = VkTyName> + Clone,
    extensions: impl Iterator<Item = VkTyName> + Clone,
) -> impl ToTokens {
    let names = features.chain(extensions);
    to_tokens_closure!(tokens {
        krs_quote_with!(tokens <-
            pub mod dependencies {
                {@*
                    #[allow(non_camel_case_types)]
                    pub trait {@names} {}
                }
            }
        )
    })
}
