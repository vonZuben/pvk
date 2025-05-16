use crate::type_conversions::convert_wrapper_from_c;
use crate::VkStr;
use crate::VkVersion;

use vk_safe_sys as vk;

use vk::context::Context;
use vk::Version;

struct_wrapper!(
/// Info about your application
///
/// The most important thing for ApplicationInfo is indicating what Vulkan Version you are targeting. Those familiar with Vulkan will
/// know you set a numerical number for the version in this structure for a specific version, or 0 to default to 1.0.0. In vk-safe,
/// you instead indicate the version you will use with [vk::instance_context].
///
/// You can optionally set you own App name / version, and Engine name / version which may be informative to the Vulkan driver.
/// From what I understand, the Vulkan driver can use your App name and version to enable App and Engine specific fixes or optimizations.
/// This is really only useful for well-known Apps and Engines (such as popular games and game engines).
ApplicationInfo<'a, C,>
impl Clone, Copy, Deref, Debug
);

impl<'a> ApplicationInfo<'a, ()> {
    /// create ApplicationInfo with a context created using [vk::instance_context]
    pub const fn new<C: Copy>(context: C) -> ApplicationInfo<'a, C>
    where
        C: Context,
        C::Commands: Version,
    {
        let _ = context;

        check_vuids::check_vuids!(ApplicationInfo);

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_apiVersion_04010: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If apiVersion is not 0, then it must be greater than or equal to VK_API_VERSION_1_0"
            }

            // Version trait will provide a proper version
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_APPLICATION_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pNext must be NULL"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_pApplicationName_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pApplicationName is not NULL, pApplicationName must be a null-terminated UTF-8"
            "string"
            }

            // ensured by VkStr
        }

        #[allow(unused_labels)]
        'VUID_VkApplicationInfo_pEngineName_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pEngineName is not NULL, pEngineName must be a null-terminated UTF-8 string"
            }

            // ensured by VkStr
        }

        let version = C::Commands::VERSION;
        unsafe {
            convert_wrapper_from_c(vk::ApplicationInfo {
                s_type: vk::StructureType::APPLICATION_INFO,
                p_next: std::ptr::null(),
                p_application_name: std::ptr::null(),
                application_version: 0,
                p_engine_name: std::ptr::null(),
                engine_version: 0,
                api_version: version.raw(),
            })
        }
    }
}

impl<'a, C> ApplicationInfo<'a, C> {
    /// Set your App name (default is nothing, which is expressed to Vulkan with a null pointer)
    pub const fn app_name(mut self, name: VkStr<'a>) -> Self {
        self.inner.p_application_name = name.as_ptr();
        self
    }

    /// Set your App version (default is zero)
    pub const fn app_version(mut self, version: VkVersion) -> Self {
        self.inner.application_version = version.raw();
        self
    }

    /// Set the name of the the Engine you are using (default is nothing, which is expressed to Vulkan with a null pointer)
    pub const fn engine_name(mut self, name: VkStr<'a>) -> Self {
        self.inner.p_engine_name = name.as_ptr();
        self
    }

    /// Set the version of the the Engine you are using (default is zero)
    pub const fn engine_version(mut self, version: VkVersion) -> Self {
        self.inner.engine_version = version.raw();
        self
    }
}
