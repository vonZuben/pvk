use std::marker::PhantomData;
use std::ffi::CStr;

use vk_safe_sys as vk;

use crate::utils::{VkVersion, OptionPtr};

/// Vulkan Instance handle
/// this struct holds all the instance level commands for both the loaded Vulkan Version (Feature)
/// and the loaded extensions
pub struct Instance<Version, Extensions> {
    /// raw handle
    handle: vk::Instance,
    /// instance commands for Version
    version: Version,
    /// extension commands for loaaded extensions
    extensions: Extensions,
}

/// Builder for creating an Instance
pub struct InstanceCreator<'a, Version, Extensions> {
    version: PhantomData<Version>,
    extensions: PhantomData<Extensions>,
    app_name: Option<&'a CStr>,
    app_version: VkVersion,
    engine_name: Option<&'a CStr>,
    engine_version: VkVersion,
}

impl<Version, Extensions> InstanceCreator<'_, Version, Extensions>
where
    Version: vk::LoadCommands + vk::version::Version,
    Extensions: vk::LoadCommands,
{
    pub fn create(&self) -> Instance<Version, Extensions> {
        let api_version: VkVersion = Version::VersionTuple.into();

        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: std::ptr::null(),
            api_version: api_version.raw(),
            p_application_name: self.app_name.as_c_ptr(),
            application_version: self.app_version.raw(),
            p_engine_name: self.engine_name.as_c_ptr(),
            engine_version: self.engine_version.raw(),
        };

        todo!();
    }
}

// impl<Version, Extensions> Default for InstanceCreator<'_, Version, Extensions>
// where
//     Version: vk::LoadCommands,
//     Extensions: vk::LoadCommands,
// {
//     fn default() -> Self {
//         Self {
//             version: Default::default(),
//             extensions: Default::default(),
//             app_name: Default::default(),
//             app_version: Default::default(),
//             engine_name: Default::default(),
//             engine_version: Default::default(),
//         }
//     }
// }

pub fn instance_creator<'a, Version: vk::LoadCommands + vk::version::Version, Extensions: vk::LoadCommands>(
) -> InstanceCreator<'a, Version, Extensions> {
    InstanceCreator {
        version: Default::default(),
        extensions: Default::default(),
        app_name: Default::default(),
        app_version: Default::default(),
        engine_name: Default::default(),
        engine_version: Default::default(),
    }
}
