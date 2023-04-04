use krs_quote::krs_quote_with;

use crate::utils::{self, VkTyName};

use crate::constants;

pub enum EnumKind {
    Normal,
    BitFlags,
}

struct ModName {
    name: String,
}

impl ModName {
    fn new(name: VkTyName) -> Self {
        let name = crate::utils::case::camel_to_snake(crate::utils::ctype_to_rtype(name.as_str())).replace("_flags", "_flag_bits");
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

pub struct EnumVariants<'a> {
    target: VkTyName,
    kind: EnumKind,
    variants: utils::VecMap<VkTyName, crate::constants::Constant3<'a>>,
}

impl<'a> EnumVariants<'a> {
    pub fn new(target: impl Into<VkTyName>, kind: EnumKind) -> Self {
        let target = target.into();
        Self {
            target,
            kind,
            variants: Default::default(),
        }
    }

    pub fn push_variant_once(&mut self, variant: constants::Constant3<'a>) {
        let name = variant.name;
        match self.variants.get(name) {
            // the vulkan spec includes redundant enum definitions
            // we only want to generate one, but we should ensure they are all consistent
            Some(already) => assert_eq!(*already, variant),
            None => self.variants.push(name, variant),
        }
    }
}

impl krs_quote::ToTokens for EnumVariants<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use crate::utils::StrAsCode;
        let target = self.target;
        let target_string = utils::ctype_to_rtype(self.target.as_str());
        let variants = self.variants.iter();

        let mod_name = ModName::new(target);

        let make_proper_name = |name| make_variant_name(target_string, utils::ctype_to_rtype(name));
        let variant_names = self.variants.iter()
            .map(|c| {
                make_proper_name(c.name.as_str()).as_code()
            });
        let variant_name_strings = self.variants.iter()
            .map(|c| {
                make_proper_name(c.name.as_str())
            });

        krs_quote_with!(tokens <-
            impl {@target} {
                {@* {@variants} }
            }
        );

        match self.kind {
            EnumKind::Normal => {
                krs_quote_with!(tokens <-
                    pub mod {@mod_name} {
                        use super::{VkEnumVariant, {@target}};
                        {@*
                            pub struct {@variant_names};
                            impl VkEnumVariant for {@variant_names} {
                                type Enum = {@target};
                                const VARIANT: i32 = {@target}::{@variant_names}.0;
                            }
                        }
                    }
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
                    pub mod {@mod_name} {
                        use super::{VkFlagBitType, VkBitmaskType, {@target}};
                        {@*
                            pub struct {@variant_names};
                            impl VkFlagBitType for {@variant_names} {
                                type FlagType = {@target};
                                const FLAG: <Self::FlagType as VkBitmaskType>::RawType = {@target}::{@variant_names}.0;
                            }
                        }
                    }

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

pub fn make_variant_name(enumeration_name: &str, variant_name: &str) -> String {
    // check for both Flags and FlagBits and remove such
    let enumeration_name = enumeration_name
        .find("FlagBits")
        .map(|i| &enumeration_name[..i])
        .unwrap_or(enumeration_name);
    let enumeration_name = enumeration_name
        .find("Flags")
        .map(|i| &enumeration_name[..i])
        .unwrap_or(enumeration_name);

    let mut enum_name = utils::case::camel_to_snake(enumeration_name);
    enum_name.make_ascii_uppercase();
    enum_name.push('_');

    let const_name_string = variant_name.replace(&enum_name, ""); //.replace("_BIT", "");

    let is_numeric = const_name_string
        .chars()
        .nth(0)
        .map(char::is_numeric)
        .unwrap_or(false);
    if is_numeric {
        format!("TYPE_{}", const_name_string)
    } else {
        const_name_string
    }
}

// pub fn make_enumeration_display_code<'a>(enums: &'a [(&String, &Vec<String>)]) -> impl Iterator<Item=TokenStream> + 'a {

//     enums.iter().map( | (enum_name, variants) | {

//         let display_cases = variants.iter().map( |enum_constant| {
//             let const_name = make_variant_name(enum_name, enum_constant).as_code();
//             quote!( Self::#const_name => Some(stringify!(#const_name)), )
//         });

//         let name = enum_name.as_code();

//         if enum_name.contains("FlagBits") || enum_name.contains("Flags") {
//             quote!{
//                 impl ::std::fmt::Display for #name {
//                     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//                         let match_flag_variant = |flags| match flags {
//                             #( #display_cases )*
//                             _ => None,
//                         };

//                         // first check if variant matches an 'ALL' flag e.g. SHADER_STAGE_ALL
//                         if let Some(display) = match_flag_variant(*self) {
//                             return write!(f, "{}", display);
//                         }

//                         // else, match and print each variant individually
//                         let mut bitset = self.0 as i32;
//                         while let Some(bit) = take_lowest_bit(&mut bitset) {
//                             let display: Option<&'static str> =
//                                 match_flag_variant( unsafe { ::std::mem::transmute::<_, Self>(bit) } );
//                             if let Some(display) = display {
//                                 if bitset == 0 {
//                                     write!(f, "{}", display)?;
//                                 }
//                                 else {
//                                     write!(f, "{} | ", display)?;
//                                 }
//                             }
//                             else {
//                                 break;
//                             }
//                         }
//                         Ok(())
//                     }
//                 }
//                 impl ::std::fmt::Debug for #name {
//                     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//                         write!(f, concat!(#enum_name, "({}):[{}]"), self.0, self)
//                     }
//                 }
//             }
//         }
//         else {
//             quote!{
//                 impl ::std::fmt::Display for #name {
//                     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//                         let display: Option<&'static str> = match *self {
//                             #( #display_cases )*
//                             _ => None,
//                         };
//                         if let Some(display) = display {
//                             write!(f, "{}", display)
//                         }
//                         else {
//                             write!(f, "")
//                         }
//                     }
//                 }
//                 impl ::std::fmt::Debug for #name {
//                     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//                         write!(f, concat!(#enum_name, "({}):[{}]"), self.0, self)
//                     }
//                 }
//             }
//         }
//     })
// }
