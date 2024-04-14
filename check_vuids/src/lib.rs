#![warn(missing_docs)]

//! Macros that are used by the check_vuids tool
//!
//! Add [`check_vuids!`] macro calls at the locations
//! where you want to check the VUIDs for a particular Vulkan item. Then use `cargo run --bin vuids` and all files
//! will be checked for check_vuids! and will ensure all VUID descriptions for the specified items are up to date by
//! automatically adding/updating [`version!`], [`description!`], and [`old_description!`] as appropriate.
//!
//! A VUID's name is represented as a block label, and the contents of the block is the VUID information.
//!
//! *The macros generate no code, and are only meant to provide information to the reader in a structural way.
//! User must manually ensure that each VUID invariant is satisfied with any appropriate means.*
//!
/*!
# Example
```
pub fn create_instance() {
    check_vuids::check_vuids!(CreateInstance);

    // with the above check_vuids! in place, the following code will be automatically generated
    // after running the check_vuids bin tool.
    #[allow(unused_labels)]
    'VUID_vkCreateInstance_ppEnabledExtensionNames_01388: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "All required extensions for each extension in the VkInstanceCreateInfo::ppEnabledExtensionNames"
        "list must also be present in that list"
        }

        // user added check
    }
}
```
*/

/// Start of a list of VUIDs to check
///
/// **DO NOT** manually use this. Use the vuids bin to automatically generate it.
#[macro_export]
macro_rules! check_vuids {
    ($name:ident) => {};
}

/// Version of the VUID to check
///
/// **DO NOT** manually use this. Use the vuids bin to automatically generate it.
#[macro_export]
macro_rules! version {
    ($ver:literal) => {};
}

/// The description of the VUID to check
///
/// **DO NOT** manually use this. Use the vuids bin to automatically generate it.
#[macro_export]
macro_rules! description {
    ($($desc:literal)*) => {};
}

/// Previous description of the VUID from before an update
///
/// This is inserted when a VUID is updated by the vuids bin so that it is easy
/// to compare and see what changed
///
/// **DO NOT** manually use this. Use the vuids bin to automatically generate it.
#[macro_export]
macro_rules! old_description {
    ($($desc:literal)*) => {};
}
