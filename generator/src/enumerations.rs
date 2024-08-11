use krs_quote::krs_quote_with;

use crate::utils::{self, VkTyName};

use crate::constants;

#[derive(Default)]
pub struct EnumVariantsCollection {
    enum_variants: utils::VecMap<utils::VkTyName, EnumVariants>,
}

pub struct Enumerations<'a>(&'a EnumVariantsCollection);

impl<'a> Enumerations<'a> {
    pub fn new(collection: &'a EnumVariantsCollection) -> Self {
        Self(collection)
    }
}

impl krs_quote::ToTokens for Enumerations<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let variants = self
            .0
            .enum_variants
            .iter()
            .filter(|ev| matches!(ev.kind, EnumKind::Normal));

        let enum_types = variants.clone().map(|ev| EnumTypes(ev));
        let enum_traits = variants.clone().map(|ev| EnumTraits(ev));

        krs_quote_with!(tokens <-
            {@* {@variants}}

            /// Type level versions of all enumeration variants
            pub mod enum_types {
                #![allow(non_camel_case_types)]

                {@* {@enum_types}}
            }

            pub mod enum_traits {
                {@* {@enum_traits}}
            }
        );
    }
}

pub struct Flags<'a>(&'a EnumVariantsCollection);

impl<'a> Flags<'a> {
    pub fn new(collection: &'a EnumVariantsCollection) -> Self {
        Self(collection)
    }
}

impl krs_quote::ToTokens for Flags<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let variants = self
            .0
            .enum_variants
            .iter()
            .filter(|ev| matches!(ev.kind, EnumKind::BitFlags));

        let flag_types = variants.clone().map(|ev| EnumTypes(ev));
        let flag_traits = variants.clone().map(|ev| FlagTraits(ev));

        krs_quote_with!(tokens <-
            {@* {@variants}}

            /// Type level versions of all Flag bits
            pub mod flag_types {
                #![allow(non_camel_case_types)]

                {@* {@flag_types}}
            }

            pub mod flag_traits {
                use std::cmp::Eq;
                use std::ops::{BitAnd, BitOr, BitXor};

                pub unsafe trait Flags<Type>: Send + Sync + Copy
                where
                    Type: BitAnd<Output = Type>
                        + BitOr<Output = Type>
                        + BitXor<Output = Type>
                        + Eq
                        + Copy
                {
                    /// Flags that **must** be included
                    const INCLUDES: Type;
                    /// Flags that **must** be excluded
                    const EXCLUDES: Type;

                    fn satisfies(flags: Type) -> bool {
                        let empty = Self::INCLUDES ^ Self::INCLUDES;
                        (Self::INCLUDES != empty)
                            && (Self::INCLUDES | flags == flags)
                            && (Self::EXCLUDES & flags == empty)
                    }
                }

                {@* {@flag_traits}}
            }
        );
    }
}

impl<'a> std::ops::Deref for EnumVariantsCollection {
    type Target = utils::VecMap<utils::VkTyName, EnumVariants>;

    fn deref(&self) -> &Self::Target {
        &self.enum_variants
    }
}

impl<'a> std::ops::DerefMut for EnumVariantsCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.enum_variants
    }
}

struct EnumTypes<'a>(&'a EnumVariants);

impl krs_quote::ToTokens for EnumTypes<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let target = self.0.target;
        let variants = self.0.variants.iter().map(|c| {
            struct Variant<'a>(&'a constants::Constant3);
            impl krs_quote::ToTokens for Variant<'_> {
                fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
                    let name = self.0.name();
                    if self.0.is_alias() {
                        let value = self.0.value();
                        krs_quote_with!(tokens <-
                            pub type {@name} = {@value};
                        )
                    } else {
                        krs_quote_with!(tokens <-
                            #[derive(Copy, Clone)]
                            pub struct {@name};
                        )
                    }
                }
            }
            Variant(c)
        });

        krs_quote_with!(tokens <-
            pub mod {@target} {
                {@* {@variants} }
            }
        )
    }
}

struct EnumTraits<'a>(&'a EnumVariants);

impl krs_quote::ToTokens for EnumTraits<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let target = self.0.target;
        let variants =
            self.0
                .variants
                .iter()
                .filter_map(|c| if c.is_alias() { None } else { Some(c.name()) });

        krs_quote_with!(tokens <-
            pub unsafe trait {@target} {
                const VALUE: crate::{@target};
            }
            {@*
                unsafe impl {@target} for crate::enum_types::{@target}::{@variants} {
                    const VALUE: crate::{@target} = crate::{@target}::{@variants};
                }
            }
        );
    }
}

struct FlagTraits<'a>(&'a EnumVariants);

impl krs_quote::ToTokens for FlagTraits<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let target = self.0.target;
        let variants =
            self.0
                .variants
                .iter()
                .filter_map(|c| if c.is_alias() { None } else { Some(c.name()) });

        krs_quote_with!(tokens <-
            pub unsafe trait {@target} : Flags<crate::{@target}> {}
            {@*
                unsafe impl Flags<crate::{@target}> for crate::flag_types::{@target}::{@variants} {
                    const INCLUDES: crate::{@target} = crate::{@target}::{@variants};
                    const EXCLUDES: crate::{@target} = crate::{@target}::empty();
                }
                unsafe impl {@target} for crate::flag_types::{@target}::{@variants} {}
            }
            unsafe impl Flags<crate::{@target}> for () {
                const INCLUDES: crate::{@target} = crate::{@target}::empty();
                const EXCLUDES: crate::{@target} = crate::{@target}::empty();
            }
            unsafe impl {@target} for () {}
        );
    }
}

pub enum EnumKind {
    Normal,
    BitFlags,
}

struct ModName {
    name: String,
}

impl ModName {
    fn new(name: VkTyName) -> Self {
        let name = crate::utils::case::camel_to_snake(crate::utils::ctype_to_rtype(name.as_str()))
            .replace("_flags", "_flag_bits");
        Self { name }
    }
}

impl krs_quote::ToTokens for ModName {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = krs_quote::Token::from(self.name.as_str());
        krs_quote_with!(tokens <-
            {@name}
        )
    }
}

pub struct EnumVariants {
    target: VkTyName,
    kind: EnumKind,
    variants: utils::VecMap<VkTyName, crate::constants::Constant3>,
}

impl EnumVariants {
    pub fn new(target: impl Into<VkTyName>, kind: EnumKind) -> Self {
        let target = target.into();
        Self {
            target,
            kind,
            variants: Default::default(),
        }
    }

    pub fn push_variant_once(&mut self, variant: constants::Constant3) {
        let name = *variant.name();
        match self.variants.get(name) {
            // the vulkan spec includes redundant enum definitions
            // we only want to generate one, but we should ensure they are all consistent
            Some(already) => assert!(already.eq(&variant)),
            None => self.variants.push(name, variant),
        }
    }
}

impl krs_quote::ToTokens for EnumVariants {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let target = self.target;
        let target_string = utils::ctype_to_rtype(self.target.as_str());
        let variants = self.variants.iter();

        let mod_name = ModName::new(target);

        let variant_name_strings = self.variants.iter().map(|c| c.name().normalize());

        let variant_names = self.variants.iter().map(|c| *c.name());

        let properties = crate::enum_properties::properties(target, variant_names.clone());

        krs_quote_with!(tokens <-
            impl {@target} {
                {@* {@variants} }
            }
            // add this to allow easily importing all names for the given type, since Rust does not currently allow importing const names from impl
            pub mod {@mod_name} {
                use super::{@target};
                {@* pub const {@variant_names}: {@target} = {@target}::{@variant_names}; }
            }

            {@properties}
        );

        match self.kind {
            EnumKind::Normal => {
                krs_quote_with!(tokens <-
                    impl std::fmt::Debug for {@target} {
                        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        let to_print = match *self {
                            {@* Self::{@variant_names} => {@variant_name_strings},}
                            _ => "Unknown Variant",
                        };
                        f.debug_tuple({@target_string})
                            .field(&to_print)
                            .finish()
                        }
                    }
                );
            }
            EnumKind::BitFlags => {
                krs_quote_with!(tokens <-
                    impl std::fmt::Debug for {@target} {
                        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        let mut self_copy = *self;
                        let to_print = std::iter::from_fn(|| self_copy.take_lowest_bit())
                            .map(|bit| {
                                match bit {
                                    {@* Self::{@variant_names} => {@variant_name_strings},}
                                    _ => "Unknown Bit",
                                }
                            })
                            .map(|s| DbgStringAsDisplay(s));
                        write!(f, "{}", {@target_string})?;
                        f.debug_list()
                            .entries(to_print)
                            .finish()
                        }
                    }
                );
            }
        }
    }
}

/// make a variant name by stripping the enumeration name from the beginning
///
/// In the Vulkan specification, enumeration variants are EnumNameVariant.
/// This would be redundant in rust to refer to variants as EnumName::EnumNameVariant.
/// Thus, we strip the EnumName from the variant.
pub fn make_variant_name(enumeration_name: &str, variant_name: &str) -> String {
    // Flags and FlagBits do not appear in the variants names
    let enumeration_name = enumeration_name
        .replace("FlagBits", "")
        .replace("Flags", "");

    // convert format to match variant format
    let mut enum_name = utils::case::camel_to_snake(&enumeration_name);
    enum_name.make_ascii_uppercase();
    enum_name.push('_');

    let const_name_string = variant_name.replace(&enum_name, "");

    let is_first_char_numeric = const_name_string
        .chars()
        .nth(0)
        .map(char::is_numeric)
        .unwrap_or(false);

    if is_first_char_numeric {
        // identifiers cannot start with a number, so prepend something
        format!("TYPE_{}", const_name_string)
    } else {
        const_name_string
    }
}
