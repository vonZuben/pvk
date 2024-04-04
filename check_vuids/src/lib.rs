#![warn(missing_docs)]

//! Macros that are used by the check_vuids tool
//!
//! Add [check_vuids!] macro calls, e.g. `check_vuids!(CreateInstance)`, at the locations
//! where you want to check the VUIDs for a particular Vulkan item. Then use `cargo run --bin check_vuids` and all files
//! will be checked for check_vuids! and will ensure all VUID descriptions for the specified items are up to date by automatically adding/updating
//! [version!], [cur_description!], and [old_description!] as appropriate.
//!
//! A VUID's name is represented as a block label, and the contents of the block is the VUID information.
//!
//! The macros only provide information to the reader. User must ensure that each VUID invariant is satisfied with any appropriate means.
//!
/*!
# Example
```
pub fn create_instance() {
    check_vuids::check_vuids!(CreateInstance);

    #[allow(unused_labels)]
    'VUID_vkCreateInstance_ppEnabledExtensionNames_01388: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
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
/// Must be placed within a block. All VUIDs to check must follow this macro call within the same block
#[macro_export]
macro_rules! check_vuids {
    ($name:ident) => {};
}

/// Version of the VUID to check
/// check_vuids will check if this is up to date with the latest version that check_vuids was compiled with
#[macro_export]
macro_rules! version {
    ($ver:literal) => {};
}

/// Current description of the VUID to check
/// check_vuids will check if this is up to date with the latest version that check_vuids was compiled with
#[macro_export]
macro_rules! cur_description {
    ($($desc:literal)*) => {};
}

/// Previous description of the VUID from before an update
/// Allows easy comparison with the updated description to see what is new
#[macro_export]
macro_rules! old_description {
    ($($desc:literal)*) => {};
}
