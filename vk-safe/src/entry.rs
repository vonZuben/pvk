use std::marker::PhantomData;
use std::ffi::CStr;
use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use crate::utils::{VkVersion, OptionPtr};

/// This is the very first point of entry that is internally used to load all ofther functions
#[link(name = "vulkan")]
extern "system" {
    #[link_name = "vkGetInstanceProcAddr"]
    fn GetInstanceProcAddr(instance: vk::Instance, p_name: *const std::os::raw::c_char)
             -> Option<vk::PFN_vkVoidFunction>;
}

/// Entry
///
/// provides a means for accessing global vulkan commands
pub struct Entry<Version> {
    commands: Version,
}

impl<Version: vk::version::Version> Entry<Version> {
    pub fn new() -> Result<Self, String> {

        let loader = |s| unsafe {
            GetInstanceProcAddr(vk::Instance{handle:std::ptr::null()}, s)
        };

        Ok(Self {
            commands: Version::load(loader)?
        })
    }
}

// safe interface for Vulkan entry level commands
//======================================
pub trait EnumerateInstanceExtensionProperties {
    fn enumerate_instance_extension_properties_len(&self, layer_name: Option<&CStr>) -> Result<u32, vk::Result>;
    fn enumerate_instance_extension_properties(&self, layer_name: Option<&CStr>) -> Result<Vec<vk::ExtensionProperties>, vk::Result>;
    fn enumerate_instance_extension_properties_user(&self, layer_name: Option<&CStr>, extensions_properties: &mut [vk::ExtensionProperties]) -> Result<(u32, vk::Result), vk::Result>;
}

impl<Version: vk::commands::EnumerateInstanceExtensionProperties> EnumerateInstanceExtensionProperties for Entry<Version> {
    fn enumerate_instance_extension_properties_len(&self, layer_name: Option<&CStr>) -> Result<u32, vk::Result> {
        let mut num = 0;
        let res;
        unsafe {
            res = self.commands.fptr()(layer_name.as_c_ptr(), &mut num, std::ptr::null_mut());
            check_raw_err!(res);
        }
        Ok(num)
    }
    fn enumerate_instance_extension_properties(&self, layer_name: Option<&CStr>) -> Result<Vec<vk::ExtensionProperties>, vk::Result> {
        let mut num = self.enumerate_instance_extension_properties_len(layer_name)?;
        let mut v = Vec::with_capacity(num as usize); // u32 as usize should always be valid
        let res;
        unsafe {
            res = self.commands.fptr()(layer_name.as_c_ptr(), &mut num, v.as_mut_ptr());
            check_raw_err!(res);
            v.set_len(num as usize);
        }
        Ok(v)
    }
    fn enumerate_instance_extension_properties_user(&self, layer_name: Option<&CStr>, extensions_properties: &mut [vk::ExtensionProperties]) -> Result<(u32, vk::Result), vk::Result> {
        let mut num = extensions_properties.len() as _;
        let res;
        unsafe {
            res = self.commands.fptr()(layer_name.as_c_ptr(), &mut num, extensions_properties.as_mut_ptr());
            check_raw_err!(res);
        }
        Ok((num, res))
    }
}

pub trait CreateInstance {
    fn create_instance(&self, create_info: &vk::InstanceCreateInfo) -> Result<vk::Instance, vk::Result>;
}

impl<Version: vk::commands::CreateInstance> CreateInstance for Entry<Version> {
    fn create_instance(&self, create_info: &vk::InstanceCreateInfo) -> Result<vk::Instance, vk::Result> {
        let mut instance = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.fptr()(create_info, None.as_c_ptr(), instance.as_mut_ptr());
            check_raw_err!(res);
            Ok(instance.assume_init())
        }
    }
}

//======================================







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
