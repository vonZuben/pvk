use generator::parse_vk_xml;

use krs_quote::krs_quote;

#[macro_use]
mod code_parts;

fn main() {
    let get_first_input_arg = || {
        let args = std::env::args_os();
        let current_exe = std::env::current_exe().unwrap();
        let current_exe = current_exe.to_string_lossy();
        for arg in args {
            let arg = arg.to_string_lossy();
            // check if the first arg is the program path
            // since this isn't technically guaranteed
            if current_exe.contains(arg.as_ref()) {
                continue;
            }
            else { // this should be the first real argument
                return arg.into_owned();
            }
        }
        panic!("no vk.xml path provided");
    };

    let code = parse_vk_xml(&get_first_input_arg());

    print!("{}", prelude());

    macro_rules! print_code_parts {
        ( $($module:ident,)* ) => {
            $( print!("{}", code.$module()); )*
        };
    }

    code_parts!(print_code_parts());
}

fn prelude() -> String {
    krs_quote!{
        macro_rules! hlist_ty {
            ($($tt:tt)*) => { () }
        }
        mod krs_hlist {
            use std::marker::PhantomData;
            pub struct hlist_ty;
            pub struct Cons<H, T>(PhantomData<H>, PhantomData<T>);
            pub struct End;
        }
        use std::ffi::*;
        fn main(){println!("Success")}
    }.to_string()
}
