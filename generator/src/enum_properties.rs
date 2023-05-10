/*!
For each Enum and Bitmask type, there is a corresponding Const trait {Enum/Bitmask}Const
The associated variants/bit implement the Const trait

in addition, there can be a Properties trait that indicates properties relevant to variant/bits
of that Enum/Bitmask type. The variants/bits implement the Properties trait in addition to the Const trait
if applicable
*/

mod format;

use krs_quote::{ToTokens, Token, TokenStream, krs_quote_with, ToTokensClosure};

use crate::utils::VkTyName;

type DynToTokens = Box<dyn ToTokens + 'static>;

pub trait EnumProperties {
    fn name(&self, name: VkTyName) -> DynToTokens;
    fn def(&self, name: VkTyName) -> DynToTokens;
    fn variant(&self, name: VkTyName, target: VkTyName) -> DynToTokens;
}

pub fn add_enum_properties(name: VkTyName, tokens: &mut TokenStream) {
    let trait_name = Token::from(format!("{name}Const"));

    let properties = get_properties(name);

    let properties_name = properties.map(|p|p.name(name));
    let properties_name = properties_name.iter(); // Option iter yields one element, so I can use as poor mans optional expansion in repeat syntax
    let properties_def = properties.map(|p|p.def(name));

    krs_quote_with!(tokens <-
        impl VkEnum for {@name} {
            fn from_variant_type<V: VkEnumVariant<Enum=Self>>(_: V) -> Self {
                Self(V::VARIANT)
            }
        }
        {@properties_def}
        pub trait {@trait_name} : VkEnumVariant<Enum={@name}> + Sized + Copy {@* + {@properties_name}} {
            fn variant(self) -> {@name} {
                <{@name} as VkEnum>::from_variant_type(self)
            }
        }
        impl<T> {@trait_name} for T where T: VkEnumVariant<Enum={@name}> + Sized + Copy {@* + {@properties_name}} {}
    );
}

pub fn variant_properties(name: VkTyName, target: VkTyName) -> impl ToTokens {
    get_properties(target).map(|p|p.variant(name, target))
}

pub fn get_properties(target: VkTyName) -> Option<&'static dyn EnumProperties> {
    match target.as_str() {
        "VkFormat" => {
            Some(&format::FormatPropertiesDef)
        }
        _ => None
    }
}