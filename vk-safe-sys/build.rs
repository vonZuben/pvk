use generator::sdk;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ERROR_MSG: &'static str = "ERROR: need to set Vulkan SDK path environment variable";

#[cfg(target_os = "windows")]
fn windows_env() -> Result<()> {
    let sdk_lib_path = sdk::sdk_path().ok_or(ERROR_MSG)?.join("Lib");
    println!("cargo:rustc-link-search={}", sdk_lib_path.display());
    Ok(())
}

fn set_env() -> Result<()> {
    println!("cargo:rerun-if-changed=../generator");

    for var in sdk::relevant_env() {
        println!("cargo:rerun-if-env-changed={var}");
    }

    #[cfg(target_os = "windows")]
    windows_env()?;

    Ok(())
}

fn main() -> Result<()> {
    generate()?;
    set_env()?;
    Ok(())
}

fn generate() -> Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").ok_or("can't get cargo 'OUT_DIR'")?;

    let vk_xml_path = sdk::vk_xml_path().ok_or(ERROR_MSG)?;
    eprintln!("{:?}", vk_xml_path);

    generator::generate_library(&out_dir, vk_xml_path)?;

    Ok(())
}
