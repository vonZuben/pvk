use std::env::var_os;

mod sdk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdk_path =
        sdk::sdk_registry_path().ok_or("error: need to set environment for Vulkan SDK")?;
    let vk_xml_path = sdk_path.join("vk.xml");
    let vuid_path = sdk_path.join("validusage.json");

    let out_path =
        var_os("TMP_LIB_DIR").ok_or("error: must set TMP_LIB_DIR for output directory")?;

    generator::generate_library(&out_path, &vk_xml_path, &vuid_path)
}
