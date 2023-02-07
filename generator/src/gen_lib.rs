/*!
program for creating setting up a vk library with all generated code
Takes vk.xml path and output directory as arguments in that order
*/
use std::{io::{Read, Write}, process::{Command, Stdio}};
use std::fs::OpenOptions;
use std::path::Path;

use krs_quote::krs_quote;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[macro_use]
mod code_parts;

/// create a file with file name 'name', and content of 'code'
fn create_file(out_dir: &str, name: &str, code: &str) -> Result<()> {
    let formatter = Command::new("rustfmt")
        .args(&["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    formatter.stdin.ok_or("Error: no stdin for rustfmt")?.write(code.as_bytes())?;

    let mut formatted_code = Vec::new();
    formatter.stdout.ok_or("Error: failed to get formatted code")?.read_to_end(&mut formatted_code)?;

    let dest_path = Path::new(out_dir).join(name);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest_path)?;

    file.write(&formatted_code)?;
    file.set_len(formatted_code.len() as _).map_err(Into::into)
}

fn make_output_directory(path: &str) -> Result<()> {
    std::fs::create_dir_all(path).map_err(Into::into)
}

fn make_lib_file(out_dir: &str) -> Result<()> {

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
        {@* mod {@module_names};}
        {@* pub use {@module_names}::*;}

        use std::ffi::{c_char, c_int, c_void, c_ulong, c_uint};
    ).to_string();

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

fn main() -> Result<()> {
    let mut args = std::env::args();
    let current_exe = std::env::current_exe().unwrap();
    let current_exe = current_exe.to_string_lossy();

    let mut get_input_arg = || {
        let arg = args.next()?;
        if current_exe.contains(&arg) {
            args.next()
        }
        else {
            Some(arg)
        }
    };

    const INPUT_ERROR_MSG: &str = "please provide vk.xml path and output directory";
    let vk_xml = get_input_arg().ok_or(INPUT_ERROR_MSG)?;
    let out_dir = get_input_arg().ok_or(INPUT_ERROR_MSG)?;

    make_output_directory(&out_dir)?;

    let code = generator::parse_vk_xml(&vk_xml);

    code_parts!(make_rs_files() code, out_dir,);

    make_lib_file(&out_dir)?;

    Ok(())
}
