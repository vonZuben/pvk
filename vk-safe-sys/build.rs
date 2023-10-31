use std::path::PathBuf;

#[cfg(target_os = "windows")]
fn windows_env(sdk_path: &str) {
    println!("cargo:rustc-link-search={sdk_path}\\Lib");
}

fn get_sdk_path() -> Result<String, &'static str> {
    let paths = [
        || std::env::var("VULKAN_SDK"),
        || std::env::var("VK_SDK_PATH"),
    ];

    for get_path in paths {
        if let Ok(path) = get_path() {
            return Ok(path);
        }
    }
    Err("Error: make sure VK_SDK_PATH or VULKAN_SDK are set properly")
}

fn set_env(#[allow(unused)] sdk_path: &str) {
    println!("cargo:rerun-if-changed=../generator");

    #[cfg(target_os = "windows")]
    windows_env(sdk_path);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdk_path = get_sdk_path()?;
    if cfg!(feature = "generate") {
        generate(&sdk_path)?;
        set_env(&sdk_path);
    }
    Ok(())
}

fn generate(sdk_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var_os("OUT_DIR").ok_or("can't get 'OUT_DIR'")?;

    let vk_xml_path: PathBuf = [sdk_path, "share", "vulkan", "registry", "vk.xml"]
        .iter()
        .collect();
    let validusage_path: PathBuf = [sdk_path, "share", "vulkan", "registry", "validusage.json"]
        .iter()
        .collect();

    generator::generate_library(&out_dir, vk_xml_path, validusage_path)
}
