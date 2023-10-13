#[cfg(target_os = "windows")]
fn target_env() {
    let vk_skd_path = std::env::var("VK_SDK_PATH");
    let vulkan_sdk = std::env::var("VULKAN_SDK");
    let options = [vk_skd_path, vulkan_sdk];

    let vk_lib_path = first_option(&options)
        .expect("Error: make sure VK_SDK_PATH or VULKAN_SDK are set properly");

    let vk_lib_path = vk_lib_path.to_owned() + "\\Lib";

    println!("cargo:rustc-link-search={}", vk_lib_path);
}

#[cfg(not(target_os = "windows"))]
fn target_env() {}

fn set_env() {
    println!("cargo:rerun-if-changed=../generator");
    target_env();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature = "generate") {
        generate()?;
        set_env();
    }
    Ok(())
}

fn generate() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var_os("OUT_DIR").ok_or("can't get 'OUT_DIR'")?;

    generator::generate_library(&out_dir, "vk.xml", "validusage.json")
}

#[cfg(target_os = "windows")]
fn first_option<T, E>(options: &[Result<T, E>]) -> Option<&T> {
    for opt in options {
        match opt {
            Ok(t) => return Some(t),
            Err(_) => {}
        }
    }
    None
}
