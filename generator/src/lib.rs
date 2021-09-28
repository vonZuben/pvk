#![recursion_limit = "1000"]

#![allow(unused)]

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
mod extensions;
mod ctype;
mod vkxml_visitor;
mod vk_parse_visitor;
mod code_generator;
mod static_code;
mod aliases;
// mod methods;

//mod take_list;

use utils::StrAsCode;

use std::path::Path;
use std::collections::HashMap;

use vkxml::*;
use proc_macro2::{TokenStream};

use definitions::*;
use enumerations::*;
use constants::*;
use commands::*;
use features::*;
use extensions::*;


pub fn generate(vk_xml_path: &str) -> String {
    // return "".to_string();
    let vk_xml_path = Path::new(vk_xml_path);
    // this it the easier to parse registry
    let registry = vk_parse::parse_file_as_vkxml(&vk_xml_path).expect("failed to parse and convert vk.xml");
    let (registry2, _) = vk_parse::parse_file(&vk_xml_path).expect("failed to parse vk.xml");

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);
    vkxml_visitor::visit_vkxml(&registry, &mut generator);

    return generator.generate_output_for_single_file();
}

