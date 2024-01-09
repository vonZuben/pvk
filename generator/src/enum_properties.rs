/*!
For each Enum and Bitmask type, there is a corresponding Const trait {Enum/Bitmask}Const
The associated variants/bit implement the Const trait

in addition, there can be a Properties trait that indicates properties relevant to variant/bits
of that Enum/Bitmask type. The variants/bits implement the Properties trait in addition to the Const trait
if applicable
*/

mod format;
mod queue;

use krs_quote::{krs_quote_with, ToTokens, TokenStream};

use crate::utils::VkTyName;

pub trait Variants: Iterator<Item = VkTyName> + Clone {}
impl<T> Variants for T where T: Iterator<Item = VkTyName> + Clone {}

struct Properties<I> {
    target: VkTyName,
    variants: I,
}

impl<I> Properties<I> {
    fn new(target: VkTyName, variants: I) -> Self {
        Self { target, variants }
    }
}

impl<I: Variants> ToTokens for Properties<I> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.target.as_str() {
            "VkFormat" => {
                format::FormatProperties::delegate_to_tokens(self, tokens);
            }
            "VkQueueFlags" => {
                queue::QueueProperties::delegate_to_tokens(self, tokens);
            }
            _ => {}
        }
    }
}

trait ToTokensDelegate<I> {
    fn delegate_to_tokens(params: &Properties<I>, tokens: &mut TokenStream);
}

pub fn properties<I: Variants>(target: VkTyName, variants: I) -> impl ToTokens {
    Properties::new(target, variants)
}
