use std::env::var_os;

mod sdk;
use sdk::{validusage_json_path, vk_xml_path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vk_xml_path =
        vk_xml_path().ok_or("ERROR: provide path for vk.xml or set path for Vulkan SDK")?;
    let vuid_path = validusage_json_path()
        .ok_or("ERROR: provide path for validusage.json or set path for Vulkan SDK")?;

    let out_path =
        var_os("TMP_LIB_DIR").ok_or("error: must set TMP_LIB_DIR for output directory")?;

    generator::generate_library(&out_path, &vk_xml_path, &vuid_path)
}
