//! Vulkan flags (also referred to as bitmasks for FlagBits)
//!
//! 🚧 docs in progress
//!
//! Flags are implemented as structs that hold a single numerical value (depends on vk.xml, currently can be u32 or u64).
//! All bits are represented as associated constants. All Flag types also has a sibling module (just a module with
//! the same name in snake_case), which also exposes all bits as standalone constants. This allows the Flag type
//! bits of `SomeFlags` to be imported with `use some_flags::*;`.
//!
//! ## Type level Flags
//!
//! Flags can be used at the type level to ensure that certain properties are available. For example, the
//! [`map_memory`](crate::vk::Device::map_memory) method requires the
//! memory to have been allocated from host visible memory that is not on a multi instance heap. This
//! is expressed with [`Includes`] and [`Excludes`] traits, which can be collectively represented
//! by a type which implements the [`Flags`](vk_safe_sys::Flags) trait.
//!
//! Use the [`flags!`] macro to create a type that correctly implements [`Flags`](vk_safe_sys::Flags), [`Includes`], and [`Excludes`].
//!
//! ## illustrative implementation example
//! ```
//! # mod example {
//! pub type Flags = u32;
//!
//! pub struct PipelineCreateFlags(pub(crate) Flags);
//!
//! impl PipelineCreateFlags {
//!     pub const DISABLE_OPTIMIZATION_BIT: Self = Self(0x00000001);
//!     pub const ALLOW_DERIVATIVES_BIT: Self = Self(0x00000002);
//!     pub const DERIVATIVE_BIT: Self = Self(0x00000004);
//!     // and more
//! }
//!
//! pub mod pipeline_create_flag_bits {
//!     use super::PipelineCreateFlags;
//!     pub const DISABLE_OPTIMIZATION_BIT: PipelineCreateFlags =
//!         PipelineCreateFlags::DISABLE_OPTIMIZATION_BIT;
//!     pub const ALLOW_DERIVATIVES_BIT: PipelineCreateFlags =
//!         PipelineCreateFlags::ALLOW_DERIVATIVES_BIT;
//!     pub const DERIVATIVE_BIT: PipelineCreateFlags = PipelineCreateFlags::DERIVATIVE_BIT;
//!     // and more
//! }
//! # }
//! ```

pub use vk_safe_sys::generated_vulkan::flag_traits;
pub use vk_safe_sys::generated_vulkan::flag_types::*;

/// Trait that represents if flag `F` is included
///
/// Use the [`flags!`](crate::flags!()) macro to create a type which implements this trait.
/// It is not recommended to manually implement this trait.
pub unsafe trait Includes<F> {}

/// Trait that represents if flag `F` is excluded
///
/// Use the [`flags!`](crate::flags!()) macro to create a type which implements this trait.
/// It is not recommended to manually implement this trait.
pub unsafe trait Excludes<F> {}

/// Create a type that represents flags that **must** be included and excluded
///
/// This will create a type with your provided name, and properly implement [`Flags`](vk_safe_sys::Flags), [`Includes`], and [`Excludes`]. Any flags
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
        #[derive(Copy, Clone)]
        $vis struct $name;

        {
            use $crate::flags::$f_type::{$($has,)* $($not,)*};

            unsafe impl $crate::flags::flag_traits::$f_type for $name {}

            type Type = $crate::raw::$f_type;

            unsafe impl $crate::flags::flag_traits::Flags<Type> for $name {
                const INCLUDES: Type = ( Type::empty() $( .or(Type::$has) )* );
                const EXCLUDES: Type = ( Type::empty() $( .or(Type::$not) )* );
            }

            $(
                unsafe impl $crate::flags::Includes<$has> for $name {}
            )*

            $(
                unsafe impl $crate::flags::Excludes<$not> for $name {}
            )*
        }
    };

    ( $f_type:ident $( + $has:ident )* $( - $not:ident )* ) => {
        {
            #[derive(Copy, Clone)]
            struct PrivateFlagsImpl;

            use $crate::flags::$f_type::{$($has,)* $($not,)*};

            unsafe impl $crate::flags::flag_traits::$f_type for PrivateFlagsImpl {}

            type Type = $crate::raw::$f_type;

            unsafe impl $crate::flags::flag_traits::Flags<Type> for PrivateFlagsImpl {
                const INCLUDES: Type = ( Type::empty() $( .or(Type::$has) )* );
                const EXCLUDES: Type = ( Type::empty() $( .or(Type::$not) )* );
            }

            $(
                unsafe impl $crate::flags::Includes<$has> for PrivateFlagsImpl {}
            )*

            $(
                unsafe impl $crate::flags::Excludes<$not> for PrivateFlagsImpl {}
            )*

            PrivateFlagsImpl
        }
    };
}
pub use flags;
