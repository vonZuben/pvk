/*!
program for creating setting up a vk library with all generated code
Takes vk.xml path and output directory as arguments in that order
*/
use std::fs::OpenOptions;
use std::path::Path;
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

use krs_quote::krs_quote;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// create a file with file name 'name', and content of 'code'
fn create_file(out_dir: &Path, name: &str, code: &str) -> Result<()> {
    let formatter = Command::new("rustfmt")
        .args(&["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    formatter
        .stdin
        .ok_or("Error: no stdin for rustfmt")?
        .write(code.as_bytes())?;

    let mut formatted_code = Vec::new();
    formatter
        .stdout
        .ok_or("Error: failed to get formatted code")?
        .read_to_end(&mut formatted_code)?;

    let dest_path = Path::new(out_dir).join(name);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest_path)?;

    file.write(&formatted_code)?;
    file.set_len(formatted_code.len() as _).map_err(Into::into)
}

fn make_output_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(Into::into)
}

fn make_lib_file(out_dir: &Path) -> Result<()> {
    macro_rules! make_module_names {
        ( $($name:ident,)* ) => {
            [ $(stringify!($name)),* ]
        };
    }

    let module_names = code_parts!(make_module_names());
    let module_names = module_names.iter().map(|m| krs_quote::Token::from(*m));

    // the first module should be 'util_code', which should be the only module to include macros
    let code = krs_quote!(
        #[macro_use]
        {@* pub mod {@module_names};}
        {@* pub use {@module_names}::*;}

        use std::ffi::{c_char, c_int, c_void, c_ulong, c_uint};
    )
    .to_string();

    create_file(out_dir, "lib.rs", &code)
}

// each file we create needs to be treated differently
// I still design this to work with the code_parts macro so that they are forced to be synced
macro_rules! make_rs_files {
    ($code:ident, $out_dir:ident, $($name:ident,)* ) => {
        $(
            let code = krs_quote::Token::from($code.$name());

            let code = krs_quote!(
                use super::*;
                {@code}
            ).to_string();

            create_file(&$out_dir, concat!(stringify!($name), ".rs"), &code)?;
        )*
    };
}

pub fn generate_library(out_dir: &Path, vk_xml: &Path) -> Result<()> {
    make_output_directory(&out_dir)?;
    let code = crate::parse_vk_xml(vk_xml);
    code_parts!(make_rs_files() code, out_dir,);
    make_lib_file(&out_dir)
}

pub fn generate_vuids_file(out_dir: &Path, validusage_json_path: &Path) -> Result<()> {
    make_output_directory(out_dir)?;
    let vuids = crate::parse_vuids(validusage_json_path);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(out_dir.join("vuids.txt"))?;

    file.write(vuids.as_bytes())?;
    file.set_len(vuids.as_bytes().len() as _)
        .map_err(Into::into)
}
