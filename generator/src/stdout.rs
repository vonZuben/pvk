use generator::generate;

use krs_quote::krs_quote;

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

    let code = generate(&get_first_input_arg());

    let code = append_main(code);

    println!("{}", code);
}

fn append_main(code: String) -> String {

    let main = krs_quote!{
        fn main(){}
    };

    format!("{}{}", code, main)
}
