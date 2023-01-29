
use std::{io::{Read, Write}, process::{Command, Stdio}};
use std::fs::OpenOptions;
use std::path::Path;

#[cfg(target_os = "windows")]
fn target_env() {
    let vk_skd_path = std::env::var("VK_SDK_PATH");
    let vulkan_sdk = std::env::var("VULKAN_SDK");
    let options = [vk_skd_path, vulkan_sdk];

    let vk_lib_path = first_option(&options).expect("Error: make sure VK_SDK_PATH or VULKAN_SDK are set properly");

    let vk_lib_path = vk_lib_path.to_owned() + "\\Lib";

    println!("cargo:rustc-link-search={}", vk_lib_path);
}

#[cfg(not(target_os = "windows"))]
fn target_env() {}

fn set_env() {
    println!("cargo:rerun-if-changed=../generator");
    target_env();
}

fn main() {
    if cfg!(feature = "generate") {
        generate();
        set_env();
    }
}

fn create_file(name: &str, code: &str) {
    let formatter = Command::new("rustfmt")
        .args(&["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Error: cannot run rustfmt");

    formatter.stdin.expect("Error: can't get rustfmt stdin").write(code.as_bytes()).expect("Error writing to rustfmt stdin");

    let mut formatted_code = Vec::new();
    formatter.stdout.expect("Error: failed to get formatted code").read_to_end(&mut formatted_code).expect("can't read from formatted code stdout");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(name);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest_path)
        .expect("Error: cannot open vk.rs for writting");

    file.write(&formatted_code).expect("Error: could not write to vk.rs");
    file.set_len(formatted_code.len() as _).expect("Error: cannot set vk.rs file len");
}

fn generate() {
    let code = generator::parse_vk_xml("vk.xml");

    create_file("util_code.rs", code.util_code());
    create_file("vulkan_traits.rs", code.vulkan_traits());
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