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
mod bitmask_traits;
mod enum_properties;

mod gen_lib;

use std::path::Path;

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

#[doc(hidden)]
/// This generates all code generated from vk.xml into a single file
/// useful for testing
///
/// used in the stdout program that is included but is just for testing
pub fn generate_output_for_single_file(vk_xml_path: impl AsRef<std::ffi::OsStr>) -> String {
    unsafe {intern::Interner::init();}
    let (registry2, _) = vk_parse::parse_file(Path::new(&vk_xml_path)).expect("failed to parse vk.xml");

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);

    return generator.generate_output_for_single_file();
}

/// Parse a xk.xml at the provided path, and provide the generated [Code]
pub fn parse_vk_xml(vk_xml_path: impl AsRef<std::ffi::OsStr>) -> Code {
    unsafe {intern::Interner::init();}
    let (registry2, _) = vk_parse::parse_file(Path::new(&vk_xml_path)).expect("failed to parse vk.xml");

    let mut generator = code_generator::Generator::default();

    vk_parse_visitor::visit_vk_parse(&registry2, &mut generator);

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
pub fn generate_library(out_dir: impl AsRef<std::ffi::OsStr>, vk_xml: impl AsRef<std::ffi::OsStr>) -> Result<(), Box<dyn std::error::Error>> {
    gen_lib::generate_library(Path::new(&out_dir), Path::new(&vk_xml))
}