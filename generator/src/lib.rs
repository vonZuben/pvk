#![recursion_limit = "1000"]

extern crate vk_parse;

#[macro_use]
mod utils;

mod intern;

mod simple_parse;

mod constants;
mod definitions;
mod enumerations;
mod commands;
mod features;
mod extensions;
mod ctype;
mod vk_parse_visitor;
mod code_generator;
mod static_code;
mod traits;

use std::path::Path;

pub fn generate(vk_xml_path: &str) -> String {
    unsafe {intern::Interner::init();}
    let vk_xml_path = Path::new(vk_xml_path);
    let (registry2, _) = vk_parse::parse_file(&vk_xml_path).expect("failed to parse vk.xml");

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);

    return generator.generate_output_for_single_file();
}

