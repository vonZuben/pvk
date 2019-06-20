
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

pub fn handle_features(features: &Features) -> TokenStream {

    let q = features.elements.iter().map(|feature| {

        //dbg!(feature);

        quote!()

    });

    quote!( #( #q )* )

}
