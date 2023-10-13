use generator::parse_vk_xml;

use krs_quote::krs_quote;

#[macro_use]
mod code_parts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    const INPUT_ERROR_MSG: &str = "please provide paths to vk.xml and validusage.json";
    let vk_xml = get_input_arg().ok_or(INPUT_ERROR_MSG)?;
    let vuid = get_input_arg().ok_or(INPUT_ERROR_MSG)?;

    let code = parse_vk_xml(vk_xml, vuid);

    print!("{}", prelude());

    macro_rules! print_code_parts {
        ( $($module:ident,)* ) => {
            $( print!("{}", code.$module()); )*
        };
    }

    code_parts!(print_code_parts());

    Ok(())
}

fn prelude() -> String {
    krs_quote!{
        use std::ffi::*;
        fn main(){println!("Success")}
    }.to_string()
}
