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

Generated code removes the "Vk" prefixes since the code can be imbedded in a crate
and used as vk to provide e.g. vk::Instance.
*/
pub struct Code {
    util_code: String,
    vulkan_traits: String,
    c_type_defs: String,
    bitmasks: String,
    structs: String,
    unions: String,
    handles: String,
    enumerations: String,
    enum_variants: String,
    function_pointers: String,
    constants: String,
    commands: String,
}

impl Code {
    /**
    Code that provided basic utility and is not based on vk.xml.
    Other parts of the generated code rely on this to compile
    */
    pub fn util_code(&self) -> &str {
        &self.util_code
    }

    /// Code that represents certain aspects of Vulkan via traits
    pub fn vulkan_traits(&self) -> &str {
        &self.vulkan_traits
    }

    /// Code for c style type definitions (just aliases for fundamental types like VkBool32)
    pub fn c_type_defs(&self) -> &str {
        &self.c_type_defs
    }

    /// Code for bitmasks
    pub fn bitmasks(&self) -> &str {
        &self.bitmasks
    }

    /// Code for structs
    pub fn structs(&self) -> &str {
        &self.structs
    }

    /// Code for unions
    pub fn unions(&self) -> &str {
        &self.unions
    }

    /// Code for handles
    pub fn handles(&self) -> &str {
        &self.handles
    }

    /// Code for enumerations
    pub fn enumerations(&self) -> &str {
        &self.enumerations
    }

    /// Code for enum_variants
    pub fn enum_variants(&self) -> &str {
        &self.enum_variants
    }

    /// Code for enumerations
    pub fn function_pointers(&self) -> &str {
        &self.function_pointers
    }

    /// Code for constants
    pub fn constants(&self) -> &str {
        &self.constants
    }

    /// Code for commands
    pub fn commands(&self) -> &str {
        &self.commands
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
        vulkan_traits: generator.vulkan_traits(),
        c_type_defs: generator.c_type_defs(),
        bitmasks: generator.bitmasks(),
        structs: generator.structs(),
        unions: generator.unions(),
        handles: generator.handles(),
        enumerations: generator.enumerations(),
        enum_variants: generator.enum_variants(),
        function_pointers: generator.function_pointers(),
        constants: generator.constants(),
        commands: generator.commands(),
    }
}
