pub mod type_conversions;

/// A specialized [`Result`] type for Vulkan operations.
///
/// This is used across the safe Vulkan interface for commands that
/// can succeed (with VK_SUCCESS) or fail (with VK_ERROR_*).
///
/// Some commands can succeed in more ways than one, and those commands
/// will handle that extra information on a case by case basis (e.g. with
/// a special type for T).
pub type Result<T> = std::result::Result<T, vk_safe_sys::Result>;