pub use crate::PFN_vkVoidFunction as VoidFunction;
pub use crate::{CommandLoadError, LoadCommands};

use crate::VkStrRaw;
use crate::{FunctionLoader, VulkanCommand};

pub trait Context {
    type Commands: LoadCommands;
}

pub unsafe trait InstanceDependencies<I, O> {}

pub unsafe trait Extensions {
    fn list_of_extensions() -> impl AsRef<[VkStrRaw]>;
}

/** define what Vulkan version and extensions will be used with an instance

### Usage
First provide the name for your context to be able to refer to it later. You can also indicate if it is `pub` to be accessible outside
the defining scope. Then pass the Version you will use, and a list of zero or more extensions all prepended with a `+`.

### Examples
```
# use vk_safe_sys::context as vk;
vk::instance_context!(pub MyInstanceContext: VERSION_1_1 + KHR_wayland_surface + KHR_surface);
vk::instance_context!(OnlyBaseVersion: VERSION_1_1);

// There are no uses for an extension only context at this time, since you cannot
// create any of the core dispatchable handles with it but it may be useful in
// future to indicate specific properties for sub-regions of your code.
vk::instance_context!(OnlyExtensions: + EXT_swapchain_colorspace + KHR_surface);
```

## Safety
Many Vulkan extensions depend on other Vulkan extensions, or base versions of Vulkan. e.g. in order to use KHR_wayland_surface, you must also use KHR_surface.
The macro generated code uses some trait implementations in order to ensure that all dependencies for each extension are present. If you fail to specify a
dependency of an extension you want to use, you will see any error such as `the trait bound `InstanceContext::commands::InstanceContext: vk_safe_sys::dependencies::traits::KHR_surface` is not satisfied`.
The last part in the path indicates that, in this example, `KHR_surface` also needs to be specified.

ℹ️ you may also see cases such as
- `VERSION_1_1__AND__VK_KHR_get_surface_capabilities2`, which means that `VERSION_1_1` (or higher) **and** `KHR_get_surface_capabilities2` must be specified
- `KHR_get_physical_device_properties2__AND__VK_KHR_surface__AND__VK_KHR_get_surface_capabilities2`, which means that `KHR_get_physical_device_properties2` **and** `KHR_surface` **and** `KHR_get_surface_capabilities2` must be specified
- `KHR_external_fence__OR__VK_VERSION_1_1`, which means `KHR_external_fence` **or** `VERSION_1_1` must be specified (this is usually because the extension got promoted to a core vulkan version,
i.e. `KHR_external_fence` was promoted to core when `VERSION_1_1` was released)

ℹ️ Some extensions get promoted to core versions. e.g. `KHR_external_fence` was promoted to core when `VERSION_1_1` was released. Thus, if you specify `VERSION_1_1`, you should not also specify
`KHR_external_fence`, or else there will be a conflict with how to load to associated commands, which is seen as a conflicting trait implementation error. i.e. If both `VERSION_1_1` and `KHR_external_fence`,
are specified, you will see something like `type annotations needed ... cannot infer type` because multiple options are available for `KHR_external_fence__OR__VK_VERSION_1_1`

ℹ️ `#[diagnostic::on_unimplemented]` should be stable soon and I plan to use it here to make better error messages.
*/
#[macro_export]
macro_rules! instance_context {
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $e_provider:ident )* ) => {
        #[allow(non_upper_case_globals)]
        $vis const $name: $name::$name = $name::$name;

        #[allow(non_snake_case)]
        pub mod $name {
            #[derive(Copy, Clone)]
            pub struct $name;

            impl $crate::context::Context for $name {
                type Commands = commands::$name;
            }

            unsafe impl $crate::CommandProvider for $name {}

            mod commands {
                $(
                    use $crate::version::instance::traits::$v_provider; // this is here so that rust analyzer auto complete can provide good suggestions see (https://blog.emi0x7d1.dev/improving-autocompletion-in-your-rust-macros/)
                    $crate::version::instance::macros::$v_provider!($name);
                    impl $crate::Version for $name {
                        const VERSION: $crate::VkVersion = $crate::VkVersion::from_triple($crate::version::numbers::$v_provider);
                    }
                )?

                $(
                    use $crate::extension::instance::traits::$e_provider; // this is here for autocomplete (see above)
                    impl $crate::dependencies::traits::$e_provider for $name {}
                    $crate::extension::instance::macros::$e_provider!($name);
                    const _ : () = {
                        $crate::dependencies::instance::$e_provider::check_dependencies(std::marker::PhantomData::<$name>)
                    };
                )*

                unsafe impl $crate::CommandProvider for $name {}

                unsafe impl $crate::context::Extensions for super::$name {
                    fn list_of_extensions() -> impl AsRef<[$crate::VkStrRaw]> {
                        use std::ffi::c_char;
                        use $crate::context::macro_helper::*;
                        let l = End;
                        $( $crate::dependencies::instance::$e_provider!(l); )*
                        l
                    }
                }

                #[allow(non_snake_case)]
                pub struct $name {
                    $( $v_provider: $crate::version::instance::structs::$v_provider, )?
                    $( $e_provider: $crate::extension::instance::structs::$e_provider, )*
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $( $v_provider: $crate::version::instance::structs::$v_provider::load(loader)?, )?
                                $( $e_provider: $crate::extension::instance::structs::$e_provider::load(loader)?, )*
                            }
                        )
                    }
                }
            }
        }
    }
}
pub use instance_context;

/** define what Vulkan version and extensions will be used with an device

This is the same as [instance_context], except for device. The Usage and Safety considerations are the same.
Some device specific example is provided below.

Extensions for a device may also depend on extensions or versions of the instance being used. These are checked
in `create_device`, which is when we know what instance / device context combination you are using. The error
may indicate that some "has_command" trait is missing, and may suggest that you need to implement an extension
trait for your instance context to address the issue.

ℹ️ `#[diagnostic::on_unimplemented]` should be stable soon and I plan to use it here to make better error messages.

### Examples
```
# use vk_safe_sys::context as vk;
vk::device_context!(pub MyDeviceContext: VERSION_1_0 + EXT_descriptor_indexing + KHR_maintenance3);
vk::device_context!(OnlyBaseVersion: VERSION_1_0);

// There are no uses for an extension only context at this time, since you cannot
// create any of the core dispatchable handles with it but it may be useful in
// future to indicate specific properties for sub-regions of your code.
vk::device_context!(OnlyExtensions: + KHR_swapchain);
```
*/
#[macro_export]
macro_rules! device_context {
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $e_provider:ident)* ) => {
        #[allow(non_upper_case_globals)]
        $vis const $name: $name::$name = $name::$name;

        #[allow(non_snake_case)]
        pub mod $name {
            #[derive(Copy, Clone)]
            pub struct $name;

            impl $crate::context::Context for $name {
                type Commands = commands::$name;
            }

            unsafe impl $crate::CommandProvider for $name {}

            mod commands {
                $(
                    use $crate::version::device::traits::$v_provider; // this is here so that rust analyzer auto complete can provide good suggestions see (https://blog.emi0x7d1.dev/improving-autocompletion-in-your-rust-macros/)
                    $crate::version::device::macros::$v_provider!($name);
                    impl $crate::Version for $name {
                        const VERSION: $crate::VkVersion = $crate::VkVersion::from_triple($crate::version::numbers::$v_provider);
                    }
                )?

                $(
                    use $crate::extension::device::traits::$e_provider; // this is here for autocomplete (see above)
                    impl $crate::dependencies::traits::$e_provider for $name {}
                    $crate::extension::device::macros::$e_provider!($name);
                    const _ : () = {
                        $crate::dependencies::device::$e_provider::check_dependencies(std::marker::PhantomData::<$name>)
                    };
                )*

                unsafe impl $crate::CommandProvider for $name {}

                #[allow(non_camel_case_types)]
                unsafe impl<I $(, $e_provider)*> $crate::context::InstanceDependencies<I, ( $($e_provider),* )> for super::$name
                    where I: $crate::CommandProvider $( + $crate::dependencies::device::$e_provider::instance::HasDependency<$e_provider> )* {}

                unsafe impl $crate::context::Extensions for super::$name {
                    fn list_of_extensions() -> impl AsRef<[$crate::VkStrRaw]> {
                        use std::ffi::c_char;
                        use $crate::context::macro_helper::*;
                        let l = End;
                        $( $crate::dependencies::device::$e_provider!(l); )*
                        l
                    }
                }

                #[allow(non_snake_case)]
                pub struct $name {
                    $( $v_provider: $crate::version::device::structs::$v_provider, )?
                    $( $e_provider: $crate::extension::device::structs::$e_provider, )*
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $( $v_provider: $crate::version::device::structs::$v_provider::load(loader)?, )?
                                $( $e_provider: $crate::extension::device::structs::$e_provider::load(loader)?, )*
                            }
                        )
                    }
                }
            }
        }
    }
}
pub use device_context;

/// The below code is for a trick to simplify the above macros
///
/// Ultimately, in the above macros, we want to build an array or slice of c-strings, which are names of extensions to enable, based on the
/// names provided by the user. However, some of the names provided by the user do not have a corresponding name to include in said list.
/// Thus, we cannot do a simple expansion of expressions (that evaluate to the name of an extension to enable), since some expressions would
/// be empty, and invalid. We can do a simple expansion to a list of items (e.g. statements), but then how do we create an array or slice
/// from a list of items?
///
/// My answer is to incrementally build a sort of list type based on the Hlist trick from crates like frunk.
/// An initial empty list is represented by End.
/// An item "T" can be appended to a list "C" by wrapping it with R<C, T>
///
/// The Len and ListOf traits ensure that a given R<C, T> (with repr(C)) can soundly be reinterpreted as a slice of T
///
/// Of course, we could just do more advanced macro tricks like incremental token munchers to more directly construct an array or slice, but this has
/// unfortunate compile time effects. I prefer to keep the macro as simple as possible to keep compile times down. The above macros can expand without any recursion.
#[doc(hidden)]
pub mod macro_helper {
    #[repr(C)]
    pub struct R<C, T>(pub C, pub T);

    #[repr(C)]
    pub struct End;

    /// provide the number of elements in the list
    pub unsafe trait Len {
        const LEN: usize;
    }

    unsafe impl Len for End {
        const LEN: usize = 0;
    }

    unsafe impl<C: Len, T> Len for R<C, T> {
        const LEN: usize = 1 + C::LEN;
    }

    /// ensure each element in the list is the same type
    pub unsafe trait ListOf<T> {}

    unsafe impl<T> ListOf<T> for End {}

    unsafe impl<C: ListOf<T>, T> ListOf<T> for R<C, T> {}

    impl End {
        pub const fn as_slice<'a, T>(&'a self) -> &'a [T] {
            &[]
        }
    }

    impl<C, T> R<C, T>
    where
        Self: ListOf<T> + Len,
    {
        pub const fn as_slice<'a>(&'a self) -> &'a [T] {
            let ptr: *const T = self as *const Self as *const T;
            unsafe { std::slice::from_raw_parts(ptr, Self::LEN) }
        }
    }

    impl<T> AsRef<[T]> for End {
        fn as_ref(&self) -> &[T] {
            self.as_slice()
        }
    }

    impl<C, T> AsRef<[T]> for R<C, T>
    where
        Self: ListOf<T> + Len,
    {
        fn as_ref(&self) -> &[T] {
            self.as_slice()
        }
    }
}
