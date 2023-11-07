use std::env::var_os;
use std::path::PathBuf;

pub fn sdk_path() -> Option<PathBuf> {
    let paths = [|| var_os("VULKAN_SDK"), || var_os("VK_SDK_PATH")];
    for get_path in paths {
        match get_path() {
            Some(path) => return Some(path.into()),
            None => {}
        }
    }

    None
}

pub fn sdk_registry_path() -> Option<PathBuf> {
    let mut path = sdk_path()?;
    path.extend(["share", "vulkan", "registry"].iter());
    Some(path)
}

pub fn vk_xml_path() -> Option<PathBuf> {
    match var_os("VK_XML_OVERRIDE") {
        Some(path) => Some(path.into()),
        None => sdk_registry_path().map(|path| path.join("vk.xml")),
    }
}

pub fn validusage_json_path() -> Option<PathBuf> {
    match var_os("VALIDUSAGE_JSON_OVERRIDE") {
        Some(path) => Some(path.into()),
        None => sdk_registry_path().map(|path| path.join("validusage.json")),
    }
}
