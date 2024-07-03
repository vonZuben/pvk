use generator::parse_vk_xml;

use krs_quote::krs_quote;

use std::env::var_os;
use std::path::Path;

use std::fs::OpenOptions;
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

#[macro_use]
mod code_parts;

mod sdk;

use generator::Generator;

use sdk::vk_xml_path;

/// This program will output the generated code to stdout, or to a single file if a file name is provided
///
/// Vulkan SDK path set in the environment to find the input files if set
/// if TMP_OUT_FILE is set in the environment, then output to the file indicated by such
/// otherwise write to stdout
fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("checking following environment variables:");
    for var in sdk::relevant_env() {
        eprintln!("{var}");
    }

    let vk_xml_path =
        vk_xml_path().ok_or("ERROR: provide path for vk.xml or set path for Vulkan SDK")?;

    let out_path = var_os("TMP_OUT_FILE");

    if out_path.is_none() {
        eprintln!("set TMP_OUT_FILE if you want to output to a file");
    }

    let code = parse_vk_xml(vk_xml_path);

    if let Some(out_path) = out_path {
        create_file(out_path.as_ref(), &code)?;
    } else {
        write_stdout(&code);
    }

    Ok(())
}

fn prelude() -> String {
    krs_quote! {
        use std::ffi::*;
        fn main(){println!("Success")}
    }
    .to_string()
}

fn write_stdout(code: &Generator) {
    print!("{}", prelude());
    macro_rules! print_code_parts {
        ( $($module:ident,)* ) => {
            $( print!("{}", code.$module()); )*
        };
    }
    code_parts!(print_code_parts());
}

/// output to file
fn create_file(out_path: &Path, code: &Generator) -> Result<(), Box<dyn std::error::Error>> {
    let formatter = Command::new("rustfmt")
        .args(&["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let prelude = prelude();
    let byte_iter = prelude.as_bytes().iter();
    macro_rules! collect_code_parts {
        ( $to:ident $($module:ident,)* ) => {
            $(
                let module = code.$module();
                let $to = $to.chain(module.as_bytes());
            )*
        };
    }
    code_parts!(collect_code_parts() byte_iter);

    // I tried writing bit by bit to the formatter, but it seems the formatter freezes when I do so
    // thus, I buffer everything and write it all at once.
    let buffer: Vec<u8> = byte_iter.copied().collect();

    formatter
        .stdin
        .ok_or("Error: no stdin for rustfmt")?
        .write(&buffer)?;

    let mut formatted_code = Vec::new();
    formatter
        .stdout
        .ok_or("Error: failed to get formatted code")?
        .read_to_end(&mut formatted_code)?;

    let dest_path = Path::new(out_path);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest_path)?;

    file.write(&formatted_code)?;
    file.set_len(formatted_code.len() as _).map_err(Into::into)
}
