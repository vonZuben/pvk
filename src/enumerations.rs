
use quote::quote;
use vkxml::*;
use proc_macro2::{TokenStream};

use crate::utils::*;

pub fn handle_enumerations(enumerations: &Enums) -> TokenStream {

    let q = enumerations.elements.iter().filter_map( |elem| match elem {
        EnumsElement::Enumeration(enm) => {
            let name = enm.name.as_code();
            //dbg!(&enm);
            let vals = enm.elements.iter().filter_map( |elem| {
                match elem {
                    EnumerationElement::Enum(enum_constant) => {
                        let const_name = enum_constant.name.as_code();
                        let val = one_option!(
                            &enum_constant.number , |num: &i32| *num as u32 ;
                            &enum_constant.hex , |hex_str| u32::from_str_radix(hex_str, 16)
                                            .expect(format!("error: enumeration hex decode error -> {}", hex_str).as_ref()) ;
                            &enum_constant.bitpos , |pos| (1 << pos) as u32 ;
                            &enum_constant.c_expression , |_| panic!("error: c_expression for enumeration val") ;
                            );

                        Some( quote!{ #const_name = #val } )
                    },
                    EnumerationElement::Notation(_notation) => None,
                    EnumerationElement::UnusedRange(_range) => None,
                }
            });

            let q = quote!{
                pub enum #name {
                    #( #vals ),*
                }
            };

            Some(q)
        },
        EnumsElement::Notation(_) => None,
    });

    quote!( #( #q )* )

}
