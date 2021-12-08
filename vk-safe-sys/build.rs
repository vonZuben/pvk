
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
    }
}

fn generate() {
    let fmt = Command::new("rustfmt")
        .args(&["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Error: cannot run rustfmt");

    fmt.stdin.expect("Error: can't get rustfmt stdin").write(generator::generate("vk.xml").as_bytes()).expect("Error writting to rustfmt stdin");

    let mut formatted_code = Vec::new();
    fmt.stdout.expect("Error: faild to get formatted code").read_to_end(&mut formatted_code).expect("can't read from formatted code stdout");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("vk.rs");

    let mut vkrs = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest_path)
        .expect("Error: cannot open vk.rs for writting");

    vkrs.write(&formatted_code).expect("Error: could not write to vk.rs");
    vkrs.set_len(formatted_code.len() as _).expect("Error: cannot set vk.rs file len");

    set_env();

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