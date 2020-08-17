
use quote::quote;
use vkxml::*;
use proc_macro2::{TokenStream};

use crate::utils::*;

pub fn make_varient_name(enumeration_name: &str, varient_name: &str) -> TokenStream {
    let ename = enumeration_name.find("FlagBits").map(|i| &enumeration_name[..i])
        .unwrap_or(enumeration_name);

    let mut enum_name = case::camel_to_snake(ename);
    enum_name.make_ascii_uppercase();
    enum_name.push('_');

    let const_name_string = varient_name.replace(&enum_name, "").replace("_BIT", "");

    let is_numeric = const_name_string.chars().nth(0).map(char::is_numeric).unwrap_or(false);
    if is_numeric {
        format!("TYPE_{}", const_name_string).as_code()
    }
    else {
        const_name_string.as_code()
    }
}

pub fn handle_enumerations<'a>(enumerations: &'a Enums, parse_state: &mut crate::ParseState<'a>) -> TokenStream {

    let q = enumerations.elements.iter().filter_map( |elem| match elem {
        EnumsElement::Enumeration(enm) => {

            match parse_state.enum_cache.insert(enm.name.as_str(), Vec::new()) {
                Some(_) => panic!("error: duplicate enum"),
                None => {}
            }

            let name = enm.name.as_code();

            // if the type is flagbits, then it was already delared
            // during definitions as bitmask
            let type_decleration =
                if enm.name.contains("FlagBits") {
                    quote!()
                }
                else {
                    quote!{
                        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
                        #[repr(transparent)]
                        pub struct #name(pub(crate) i32);
                    }
                };

            let vals = enm.elements.iter().filter_map( |elem| {
                match elem {
                    EnumerationElement::Enum(enum_constant) => {

                        // insert variants into hashmap
                        parse_state.enum_cache.get_mut(enm.name.as_str())
                            .expect("error: enum not in cahce")
                            .push(enum_constant.name.as_str());

                        let const_name = make_varient_name(enm.name.as_str(), enum_constant.name.as_str());

                        let val = one_option!(

                            &enum_constant.number , |num: &i32| num.to_string().as_code() ;
                            &enum_constant.hex , |hex_str| format!("0x{:0>8}", hex_str).as_code() ;
                            &enum_constant.bitpos , |bitpos: &u32| format!("0x{:0>8X}", (1u32 << bitpos)).as_code() ;
                            &enum_constant.c_expression , |_| panic!("error: c_expression for enumeration val") ;

                            );

                        Some( quote!{ pub const #const_name: Self = #name(#val); } )
                    },
                    EnumerationElement::Notation(_notation) => None,
                    EnumerationElement::UnusedRange(_range) => None,
                }
            });

            // TODO add code to safely create from raw
            //
            //let possible_varients = enm.elements.iter().filter_map( |elem| {
            //    match elem {
            //        EnumerationElement::Enum(enum_constant) => {
            //            let const_name = make_varient_name(&enm, &enum_constant);
            //            Some( quote!( Self::#const_name => true, ) )
            //        },
            //        EnumerationElement::Notation(_notation) => None,
            //        EnumerationElement::UnusedRange(_range) => None,
            //    }
            //});

            //let to_from_raw_code = if enm.name.contains("FlagBits") {
            //    impl TryFrom<Flags> for #name {
            //        type Error = &'static str;
            //        fn try_from(val: Flags) -> Result<Self, Self::Error> {
            //            if let Some(valid) = match Self(val)
            //        }
            //    }
            //}
            //else {
            //};

            let q = quote!{
                #type_decleration
                impl #name {
                    #( #vals )*
                }
            };

            Some(q)
        },
        EnumsElement::Notation(_) => None,
    });

    quote!( #( #q )* )

}

pub fn make_enumeration_display_code<'a>(parse_state: &'a crate::ParseState) -> impl Iterator<Item=TokenStream> + 'a {

    parse_state.enum_cache.iter().map( | (enum_name, variants) | {

        let display_cases = variants.iter().map( |enum_constant| {
            let const_name = make_varient_name(enum_name, enum_constant);
            quote!( Self::#const_name => Some(stringify!(#const_name)), )
        });

        let name = enum_name.as_code();

        if enum_name.contains("FlagBits") {
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
            }
        }
    })
}
