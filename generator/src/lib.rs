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

#[macro_use]
mod code_parts;

mod intern;

mod simple_parse;

mod code_generator;
mod commands;
mod constants;
mod ctype;
mod definitions;
mod enum_properties;
mod enumerations;
mod extensions;
mod features;
mod static_code;
mod traits;
mod vk_parse_visitor;
mod vuid;
mod vuid_visitor;

mod gen_lib;

use std::fs::File;
use std::io::Read;
use std::{ffi::OsStr, path::Path};

macro_rules! make_code_type {
    ( $($param:ident,)* ) => {
        /**
        The generated code

        Provides methods for obtaining different parts of the code as &str

        The generated code is mostly unformatted (there are newlines in some places to make
        it easier to read even without rustfmt; based on how [krs_quote] works). However,
        running rust fmt is still recommended if the output is for human.

        Generated code removes the "Vk" prefixes since the code can be imbedded in a crate
        and used as vk to provide e.g. vk::Instance.
        */
        pub struct Code {
            $($param: String),*
        }

        impl Code {
            $(
                /// get subject code part
                pub fn $param(&self) -> &str {
                    &self.$param
                }
            )*
        }
    };
}

code_parts!(make_code_type(;));

/// Parse a xk.xml at the provided path, and provide the generated [Code]
pub fn parse_vk_xml(vk_xml_path: impl AsRef<OsStr>, vuid_path: impl AsRef<OsStr>) -> Code {
    unsafe {
        intern::Interner::init();
    }

    // vk_xml registry
    let (registry2, _) =
        vk_parse::parse_file(Path::new(&vk_xml_path)).expect("failed to parse vk.xml");

    // vuids
    let mut vuid_json_string = String::new();
    File::open(vuid_path.as_ref())
        .expect("failed to open vuid file")
        .read_to_string(&mut vuid_json_string)
        .expect("failed to read vuid file");
    let vuid_json_parser = vuid_visitor::VuidJsonStrParser::new(&vuid_json_string);

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);
    vuid_visitor::visit_vuids(vuid_json_parser, &mut generator);

    macro_rules! get_code_parts {
        ( $generator:ident $($param:ident,)* ) => {
            Code {
                $( $param: $generator.$param(), )*
            }
        };
    }

    code_parts!(get_code_parts() generator)
}

/// generate all the code parts into files that can be used for the src directory of a standalone crate
/// or can be embedded into another crate
pub fn generate_library(
    out_dir: impl AsRef<OsStr>,
    vk_xml: impl AsRef<OsStr>,
    vuid: impl AsRef<OsStr>,
) -> Result<(), Box<dyn std::error::Error>> {
    gen_lib::generate_library(Path::new(&out_dir), Path::new(&vk_xml), Path::new(&vuid))
}
