use std::env::var_os;
use std::path::PathBuf;

type EnvironmentVarName = &'static str;

const VULKAN_SDK: EnvironmentVarName = "VULKAN_SDK";
const VK_SDK_PATH: EnvironmentVarName = "VK_SDK_PATH";
const VK_XML_OVERRIDE: EnvironmentVarName = "VK_XML_OVERRIDE";
const VALIDUSAGE_JSON_OVERRIDE: EnvironmentVarName = "VALIDUSAGE_JSON_OVERRIDE";

/**
Provides an iterator over all environment variables that are relevant for generating the Vulkan code

This is useful in a build script for setting cargo:rerun-if-env-changed=NAME
 */
#[allow(unused)]
pub fn relevant_env() -> impl Iterator<Item = &'static str> {
    [
        VK_XML_OVERRIDE,
        VALIDUSAGE_JSON_OVERRIDE,
        VULKAN_SDK,
        VK_SDK_PATH,
    ]
    .iter()
    .copied()
}

/**
Provide path the the Vulkan SDK based on the standard environment variables
 */
pub fn sdk_path() -> Option<PathBuf> {
    let paths = [|| var_os(VULKAN_SDK), || var_os(VK_SDK_PATH)];
    for get_path in paths {
        match get_path() {
            Some(path) => return Some(path.into()),
            None => {}
        }
    }

    None
}

/**
Provide path to the registry folder in the Vulkan SDK

This is the folder where files like "vk.xml" should be provided
 */
pub fn sdk_registry_path() -> Option<PathBuf> {
    let mut path = sdk_path()?;
    path.extend(["share", "vulkan", "registry"].iter());
    Some(path)
}

/**
Provide path to vk.xml

Defaults to the file in the registry folder of the Vulkan SDK.

can be overridden by setting VK_XML_OVERRIDE to the desired file path
 */
pub fn vk_xml_path() -> Option<PathBuf> {
    match var_os(VK_XML_OVERRIDE) {
        Some(path) => Some(path.into()),
        None => sdk_registry_path().map(|path| path.join("vk.xml")),
    }
}

/**
Provide path to validusage.json

Defaults to the file in the registry folder of the Vulkan SDK.

can be overridden by setting VALIDUSAGE_JSON_OVERRIDE to the desired file path
 */
pub fn validusage_json_path() -> Option<PathBuf> {
    match var_os(VALIDUSAGE_JSON_OVERRIDE) {
        Some(path) => Some(path.into()),
        None => sdk_registry_path().map(|path| path.join("validusage.json")),
    }
}
