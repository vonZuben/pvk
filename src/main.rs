#![recursion_limit = "200"]

use quote::quote;

// just for coverting the xml file into a vkxml registry
extern crate vk_parse;

#[macro_use]
mod utils;
mod constants;
mod definitions;
mod enumerations;

use std::path::Path;

use vkxml::*;
use proc_macro2::{TokenStream};

use definitions::*;
use enumerations::*;
use constants::*;

pub fn vkxml_registry_token_stream(reg_elem: &vkxml::RegistryElement) -> TokenStream {
    match reg_elem {
        RegistryElement::Definitions(definition) => {
            handle_definitions(definition)
        },
        RegistryElement::Constants(cnts) => {
            handle_constants(cnts)
        },
        RegistryElement::Enums(enums) => {
            handle_enumerations(enums)
        }
        _ => quote!(),
    }
}

fn main() {
    let registry = vk_parse::parse_file_as_vkxml(Path::new("vk.xml"));

    let tokens = registry.elements.iter().map(vkxml_registry_token_stream);

    let platform_specific_types = utils::platform_specific_types();

    let q = quote!{
        #platform_specific_types
        #(#tokens)*
    };
    //dbg!(registry);
    println!("use std::os::raw::*;{}", q);

}

