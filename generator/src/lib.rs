#![warn(missing_docs)]

/*!
This crate is for parsing vk.xml and generating code intended for use in another
crate that provides a safe vulkan interface that is as close as possible to using bare
vulkan, with a few nice rust additions.

These docs are still very work in progress.
*/

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

/**
The generated code

Provides methods for obtaining different parts of the code as &str

The generated code is mostly unformatted (there are newlines in some places to make
it easier to read even without rustfmt; based on how [krs_quote] works). However,
running rust fmt is still recommended if the output is for human.
*/
pub struct Code {
    util_code: String,
}

impl Code {
    /// Code that provided basic utility and is not based on vk.xml
    /// Other parts of the generated code rely on this to compile
    pub fn util_code(&self) -> &str {
        &self.util_code
    }
}

#[deprecated]
/// This generates all code generated from vk.xml into a single file
pub fn generate(vk_xml_path: &str) -> String {
    unsafe {intern::Interner::init();}
    let vk_xml_path = Path::new(vk_xml_path);
    let (registry2, _) = vk_parse::parse_file(&vk_xml_path).expect("failed to parse vk.xml");

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);

    return generator.generate_output_for_single_file();
}

/// Parse a xk.xml at the provided path, and provide the generated [Code]
pub fn parse_vk_xml(vk_xml_path: &str) -> Code {
    unsafe {intern::Interner::init();}
    let vk_xml_path = Path::new(vk_xml_path);
    let (registry2, _) = vk_parse::parse_file(&vk_xml_path).expect("failed to parse vk.xml");

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);

    Code {
        util_code: generator.static_code(),
    }
}