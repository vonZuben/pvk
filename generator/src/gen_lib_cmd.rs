type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

    generator::generate_library(&out_dir, &vk_xml)
}