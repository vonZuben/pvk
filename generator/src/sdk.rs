use std::path::PathBuf;

pub fn sdk_path() -> Option<PathBuf> {
    use std::env::var_os;

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
