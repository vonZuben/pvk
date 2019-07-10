#![recursion_limit = "200"]

use quote::quote;

// just for coverting the xml file into a vkxml registry
extern crate vk_parse;

#[macro_use]
mod utils;
mod constants;
mod definitions;
mod enumerations;
mod commands;
mod features;

mod removable_linked_list;

use removable_linked_list as rll;

use std::path::Path;

use vkxml::*;
use proc_macro2::{TokenStream};

use definitions::*;
use enumerations::*;
use constants::*;
use commands::*;
use features::*;

// keep certain mutable state while parsing the registry
pub struct ParseState {
}

pub fn vkxml_registry_token_stream(reg_elem: &vkxml::RegistryElement, parse_state: &mut ParseState) -> TokenStream {
    match reg_elem {
        RegistryElement::Definitions(definition) => {
            handle_definitions(definition, parse_state)
        },
        RegistryElement::Constants(cnts) => {
            handle_constants(cnts)
        },
        RegistryElement::Enums(enums) => {
            handle_enumerations(enums)
        }
        RegistryElement::Commands(cmds) => {
            handle_commands(cmds)
        }
        RegistryElement::Features(features) => {
            handle_features(features)
        }
        _ => quote!(),
    }
}

fn main() {
    // this it the easier to parse registry
    let registry = vk_parse::parse_file_as_vkxml(Path::new("vk.xml"));

    // this registry is closer to the xml formate, but it sucks to parse
    // but it does include the aliases
    let registry2 = vk_parse::parse_file(Path::new("vk.xml"));

    let mut command_list: rll::RList<_> = rll::RList::new();
    command_list.add(5);

    let mut parse_state = ParseState {};

    //println!("{:#?}", registry2);

    let tokens = registry.elements.iter().map(|relem| vkxml_registry_token_stream(relem, &mut parse_state));

    let aliases = registry2
        .0
        .iter()
        .filter_map(|item| match item {
            vk_parse::RegistryChild::Types(ref ty) => {
                Some(generate_aliases_of_types(ty))
            }
            _ => None,
        });

    let allow_vulkan_name_formats = quote!{
        #![allow(non_camel_case_types)]
        #![allow(non_snake_case)]
        #![allow(non_upper_case_globals)]
        #![allow(unused)]
    };

    let initial_test_code = quote!{
        use std::os::raw::*;
        fn main(){}
    };

    let platform_specific_types = utils::platform_specific_types();

    let q = quote!{
        #allow_vulkan_name_formats
        #initial_test_code
        #platform_specific_types
        #(#tokens)*
        #(#aliases)*
    };

    println!("{}", q);

}

