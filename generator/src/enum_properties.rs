/*!
For each Enum and Bitmask type, there is a corresponding Const trait {Enum/Bitmask}Const
The associated variants/bit implement the Const trait

in addition, there can be a Properties trait that indicates properties relevant to variant/bits
of that Enum/Bitmask type. The variants/bits implement the Properties trait in addition to the Const trait
if applicable
*/

mod format;

use krs_quote::{krs_quote_with, ToTokens, TokenStream};

use crate::utils::VkTyName;

struct Properties<I> {
    target: VkTyName,
    variants: I,
}

impl<I> Properties<I> {
    fn new(target: VkTyName, variants: I) -> Self {
        Self { target, variants }
    }
}

impl<I: Iterator<Item = VkTyName> + Clone> ToTokens for Properties<I> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.target.as_str() {
            "VkFormat" => {
                format::FormatProperties::delegate_to_tokens(self, tokens);
            }
            _ => {}
        }
    }
}

trait ToTokensDelegate<I> {
    fn delegate_to_tokens(params: &Properties<I>, tokens: &mut TokenStream);
}

pub fn properties<I: Iterator<Item = VkTyName> + Clone>(
    target: VkTyName,
    variants: I,
) -> impl ToTokens {
    Properties::new(target, variants)
}
