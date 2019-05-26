
use quote::quote;
use vkxml::*;
use proc_macro2::{TokenStream};

use crate::utils::*;

pub fn handle_enumerations(enumerations: &Enums) -> TokenStream {

    let q = enumerations.elements.iter().filter_map( |elem| match elem {
        EnumsElement::Enumeration(enm) => {

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
                        let const_name = enum_constant.name.as_code();
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
