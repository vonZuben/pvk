//! Vulkan flags (also referred to as bitmasks for FlagBits)
//!
//! 🚧 docs in progress

pub use vk_safe_sys::generated_vulkan::bitmask_variants::*;
pub use vk_safe_sys::generated_vulkan::bitmasks::*;

/** Trait for representing bit flags

Use the [`flags!`](crate::flags!()) macro to create a type which implements this trait.
It is not recommended to manually implement this trait.

A type which implements this trait represents flags that **must** be **included** and **excluded**.
Please check the documentation for APIs that require a `Flags` implementor.

This trait is unsafe to implement because it **must** also be implemented consistently with [`Includes`] and [`Excludes`] traits.
 */
pub unsafe trait Flags {
    /// The specific type of flags (e.g. [MemoryPropertyFlags]
    type Type;
    /// Flags that **must** be **included**
    const INCLUDES: Self::Type;
    /// Flags that **must** be **excluded**
    const EXCLUDES: Self::Type;
}

/// Trait that represents if flag `F` is **included**
///
/// Use the [`flags!`](crate::flags!()) macro to create a type which implements this trait.
/// It is not recommended to manually implement this trait.
pub unsafe trait Includes<F>: Flags {}

/// Trait that represents if flag `F` is **excluded**
///
/// Use the [`flags!`](crate::flags!()) macro to create a type which implements this trait.
/// It is not recommended to manually implement this trait.
pub unsafe trait Excludes<F>: Flags {}

/// Create a type that represents flags that **must** be **included** and **excluded**.
///
/// This will create a type with your provided name, and properly implement [`Flags`], [`Includes`], and [`Excludes`]. Any flags
/// not specified as included or excluded may or may not actually be included, and are considered unknown or don't care.
///
/// *zero or more `+` includes must come before zero or more `-` excludes*
///
/// ## Example
/// ```
/// use vk_safe as vk;
///
/// // These are memory type properties and heap properties that are needed for memory to be mappable
///
/// // memory type **must** include HOST_VISIBLE_BIT
/// vk::flags!(MyMemoryProperties: MemoryPropertyFlags + HOST_VISIBLE_BIT);
/// // memory heap **must not** include MULTI_INSTANCE_BIT
/// vk::flags!(MyHeapProperties: MemoryHeapFlags - MULTI_INSTANCE_BIT);
///
/// // Example of including both positive and negative
/// vk::flags!(pub Random: MemoryPropertyFlags + HOST_VISIBLE_BIT - HOST_CACHED_BIT);
/// ```
#[macro_export]
macro_rules! flags {
    ( $vis:vis $name:ident : $f_type:ident $( + $has:ident )* $( - $not:ident )* ) => {
        $vis struct $name;

        {
            use vk_safe_sys::flag_types::$f_type;
            $( use $f_type::$has; )*
            $( use $f_type::$not; )*

            unsafe impl $crate::flags::Flags for $name {
                type Type = vk_safe_sys::$f_type;
                const INCLUDES: Self::Type = ( Self::Type::empty() $( .or(Self::Type::$has) )* );
                const EXCLUDES: Self::Type = ( Self::Type::empty() $( .or(Self::Type::$not) )* );
            }

            $(
                unsafe impl $crate::flags::Includes<$has> for $name {}
            )*

            $(
                unsafe impl $crate::flags::Excludes<$not> for $name {}
            )*
        }
    };
}
pub use flags;
