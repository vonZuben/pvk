use generator::sdk;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ERROR_MSG: &'static str = "ERROR: need to set Vulkan SDK path environment variable";

fn set_env() -> Result<()> {
    println!("cargo:rerun-if-changed=../generator");

    for var in sdk::relevant_env() {
        println!("cargo:rerun-if-env-changed={var}");
    }

    Ok(())
}

fn main() -> Result<()> {
    generate()?;
    set_env()?;
    Ok(())
}

fn generate() -> Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").ok_or("can't get cargo 'OUT_DIR'")?;

    let validusage_path = sdk::validusage_json_path().ok_or(ERROR_MSG)?;
    eprintln!("{:?}", validusage_path);

    generator::generate_vuids_file(out_dir, validusage_path)?;

    Ok(())
}
