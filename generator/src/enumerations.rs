
use quote::{quote, ToTokens};
use vkxml::*;
use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::utils;

use crate::constants;

pub struct EnumVariants<'a> {
    target: &'a str,
    variants: utils::VecMap<&'a str, crate::constants::Constant2<'a>>,
}

impl<'a> EnumVariants<'a> {
    pub fn new(target: &'a str) -> Self {
        Self {
            target,
            variants: Default::default(),
        }
    }

    pub fn extend_variants(&mut self, variants: impl IntoIterator<Item=constants::Constant2<'a>>) {
        for variant in variants.into_iter() {
            self.push_variant_once(variant);
        }
    }

    pub fn push_variant_once(&mut self, variant: constants::Constant2<'a>) {
        let name = variant.name;
        match self.variants.get(name) {
            // the vulkan spec includes redundant enum definitions
            // we only want to generate one, but we should ensure they are all consistent
            Some(already) => assert_eq!(*already, variant),
            None => self.variants.push(name, variant),
        }
    }
}

impl ToTokens for EnumVariants<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;
        let target = self.target.as_code();
        let variants = self.variants.iter();
        quote!(
            impl #target {
                #(#variants)*
            }
        ).to_tokens(tokens);
    }
}

pub fn make_variant_name(enumeration_name: &str, varient_name: &str) -> String {
    let extension_tags = std::collections::HashMap::<&str,&str>::new();

    // all tags are upper case
    // find the "last" lower case letter and then assume that anything following might be a tag
    let maybe_tag = enumeration_name.my_rfind(char::is_lowercase)
        .and_then(|i| {
            if enumeration_name.len() == i + 1 {
                None
            }
            else {
                Some(&enumeration_name[i+1..])
            }
        });

    // ensure that the maybe_tag is in fact a tag
    // and then remove it from the enumeration_name
    let enumeration_name = maybe_tag.filter(|tag| extension_tags.contains_key(tag))
        .map(|tag| &enumeration_name[..enumeration_name.len()-tag.len()])
        .unwrap_or(enumeration_name);

    // check for both Flags and FlagBits and remove such
    let enumeration_name = enumeration_name.find("FlagBits").map(|i| &enumeration_name[..i])
        .unwrap_or(enumeration_name);
    let enumeration_name = enumeration_name.find("Flags").map(|i| &enumeration_name[..i])
        .unwrap_or(enumeration_name);

    let mut enum_name = case::camel_to_snake(enumeration_name);
    enum_name.make_ascii_uppercase();
    enum_name.push('_');

    let const_name_string = varient_name.replace(&enum_name, "").replace("_BIT", "");

    let is_numeric = const_name_string.chars().nth(0).map(char::is_numeric).unwrap_or(false);
    if is_numeric {
        format!("TYPE_{}", const_name_string)
    }
    else {
        const_name_string
    }
}


pub fn make_enumeration_display_code<'a>(enums: &'a [(&String, &Vec<String>)]) -> impl Iterator<Item=TokenStream> + 'a {

    enums.iter().map( | (enum_name, variants) | {

        let display_cases = variants.iter().map( |enum_constant| {
            let const_name = make_variant_name(enum_name, enum_constant).as_code();
            quote!( Self::#const_name => Some(stringify!(#const_name)), )
        });

        let name = enum_name.as_code();

        if enum_name.contains("FlagBits") || enum_name.contains("Flags") {
            quote!{
                impl ::std::fmt::Display for #name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        let match_flag_varient = |flags| match flags {
                            #( #display_cases )*
                            _ => None,
                        };

                        // first check if variant matches an 'ALL' flag e.g. SHADER_STAGE_ALL
                        if let Some(disp) = match_flag_varient(*self) {
                            return write!(f, "{}", disp);
                        }

                        // else, match and print each variant individually
                        let mut bitset = self.0 as i32;
                        while let Some(bit) = take_lowest_bit(&mut bitset) {
                            let disp: Option<&'static str> =
                                match_flag_varient( unsafe { ::std::mem::transmute::<_, Self>(bit) } );
                            if let Some(disp) = disp {
                                if bitset == 0 {
                                    write!(f, "{}", disp)?;
                                }
                                else {
                                    write!(f, "{} | ", disp)?;
                                }
                            }
                            else {
                                break;
                            }
                        }
                        Ok(())
                    }
                }
                impl ::std::fmt::Debug for #name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        write!(f, concat!(#enum_name, "({}):[{}]"), self.0, self)
                    }
                }
            }
        }
        else {
            quote!{
                impl ::std::fmt::Display for #name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        let disp: Option<&'static str> = match *self {
                            #( #display_cases )*
                            _ => None,
                        };
                        if let Some(disp) = disp {
                            write!(f, "{}", disp)
                        }
                        else {
                            write!(f, "")
                        }
                    }
                }
                impl ::std::fmt::Debug for #name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        write!(f, concat!(#enum_name, "({}):[{}]"), self.0, self)
                    }
                }
            }
        }
    })
}
