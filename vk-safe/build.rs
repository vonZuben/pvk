use vk_safe_sys::features_and_extensions;

fn main() {
    for fe in features_and_extensions() {
        println!("cargo::rustc-cfg={}", fe);
        println!("cargo::rustc-check-cfg=cfg({})", fe);
    }
}
