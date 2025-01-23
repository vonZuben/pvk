#![warn(missing_docs)]

/*!
Generate a raw unsafe rust interface based on the definitions in vk.xml
*/

extern crate vk_parse;

#[macro_use]
mod utils;

#[macro_use]
mod code_parts;

mod intern;

mod simple_parse;

mod code_generator;
mod commands;
mod constants;
mod ctype;
mod dependencies;
mod enum_properties;
mod enumerations;
mod extensions;
mod features;
mod static_code;
mod traits;
mod types;
mod vk_parse_visitor;
mod vuid;
mod vuid_visitor;
mod vulkansc;

mod gen_lib;

mod vuid_generator;

/**
Provide standard interface for finding vulkan definition files

When generating the Vulkan rust code, certain Vulkan Docs files are needed.
These files are provided by the Vulkan SDK. It is a good idea to have the Vulkan SDK installed
when developing a Vulkan App, but it is also possible to override the paths to the needed files.

See module functions for more details.
*/
pub mod sdk;

pub use utils::VecMap;

pub use code_generator::Generator;

use std::fs::File;
use std::io::Read;
use std::{ffi::OsStr, path::Path};

/// Parse a xk.xml at the provided path, and provide the Code Generator
pub fn parse_vk_xml(vk_xml_path: impl AsRef<Path>) -> Generator {
    unsafe {
        intern::Interner::init();
    }

    // vk_xml registry
    let (registry2, _) =
        vk_parse::parse_file(vk_xml_path.as_ref()).expect("failed to parse vk.xml");

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);

    generator
}

/// Parse validusage.json at provided path and provide the generated code for vuid checks
pub fn parse_vuids(vuid_path: impl AsRef<Path>) -> String {
    unsafe {
        intern::Interner::init();
    }

    // vuids
    let mut vuid_json_string = String::new();
    File::open(vuid_path)
        .expect("failed to open vuid file")
        .read_to_string(&mut vuid_json_string)
        .expect("failed to read vuid file");
    let vuid_json_parser = vuid_visitor::VuidJsonStrParser::new(&vuid_json_string);

    let mut generator = vuid_generator::VuidGenerator::default();

    vuid_visitor::visit_vuids(vuid_json_parser, &mut generator);

    generator.vuids()
}

/// generate all the code parts into files that can be used for the src directory of a standalone crate
/// or can be embedded into another crate
pub fn generate_library(
    out_dir: impl AsRef<OsStr>,
    vk_xml: impl AsRef<OsStr>,
) -> Result<(), Box<dyn std::error::Error>> {
    let code = parse_vk_xml(vk_xml.as_ref());
    let path = Path::new(&out_dir);
    gen_lib::generate_library(path, &code)?;
    gen_lib::generate_feature_and_extension_list(path, &code)?;
    Ok(())
}

/// generate vuids in provided directory, by parsing provided validusage.json file
pub fn generate_vuids_file(
    out_dir: impl AsRef<Path>,
    validusage_json_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    gen_lib::generate_vuids_file(out_dir.as_ref(), validusage_json_path.as_ref())
}
