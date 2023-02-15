#[cfg(not(feature = "generate"))]
#[macro_use]
mod pre_built;

#[cfg(not(feature = "generate"))]
pub use pre_built::*;

#[cfg(feature = "generate")]
#[macro_use]
mod generated {
    include!{concat!(env!("OUT_DIR"), "/lib.rs")}

    // temp
    impl VulkanExtension for () {
        type Require = krs_hlist::hlist_ty!();

        const VK_NAME: *const c_char = b"\0".as_ptr().cast();

        type ExtensionType = krs_hlist::hlist_ty!();

        type InstanceCommands = krs_hlist::hlist_ty!();

        type DeviceCommands = krs_hlist::hlist_ty!();
    }
}

#[cfg(feature = "generate")]
pub use generated::*;