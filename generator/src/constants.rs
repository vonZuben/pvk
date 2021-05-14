
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

pub fn handle_constants(constants: &Constants) -> TokenStream {

    let q = constants.elements.iter().map(|constant| {
        let name = constant.name();
        let ty = constant.ty();
        let val = constant.val();
        quote!( pub const #name: #ty = #val; )
    });

    quote!( #( #q )* )

}

trait ConstExt {
    fn ty(&self) -> TokenStream;
    fn val(&self) -> TokenStream;
    fn name(&self) -> TokenStream;
}

impl ConstExt for vkxml::Constant {

    fn ty(&self) -> TokenStream {
        if self.name.contains("TRUE") || self.name.contains("FALSE") {
            return quote!(Bool32);
        }
        else { one_option!(

                &self.number , |_| quote!(usize) ;

                &self.hex , |_| quote!(usize) ;

                &self.bitpos , |_| panic!("error: trying to get bitpos type not implemented -> {}", self.name) ;

                &self.c_expression , |expr: &str| {
                    match &expr {
                        e if e.contains("ULL") => quote!(u64),
                        e if e.contains("U") => quote!(u32),
                        e if e.contains("f") => quote!(f32),
                        _ => quote!(usize),
                    }
                }
                )
        }

    }
    fn val(&self) -> TokenStream {
        one_option!(

            &self.number , |num: &i32| { num.to_string().as_code() } ;

            &self.hex , |hex_str| format!("0x{:0>8}", hex_str).as_code() ;

            &self.bitpos , |_| panic!("error: trying to take bit pos for constant not implemented") ;

            &self.c_expression , |expr: &String| {
                let v = expr
                    .replace("ULL", "")
                    .replace("U", "")
                    .replace("~", "!")
                    .replace("f", "")
                    .as_code();
                quote!(#v)

            } ;
        )
    }
    fn name(&self) -> TokenStream {
        self.name.as_code()
    }
}
