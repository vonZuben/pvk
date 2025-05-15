use std::marker::PhantomData;

use crate::type_conversions::{ConvertWrapper, Wrapper};

use vk_safe_sys as vk;

use vk::{BaseInStructure, BaseOutStructure};

// #[doc(hidden)]
// pub unsafe trait Pnext<S>: vk::StructExtends<S> {
//     /// link all members in the p_next chain
//     ///
//     /// By taking a reference to the struct, we can ensure that it
//     /// remains in place for the length of the borrow (baring any kind of internal mutability)
//     /// Thus, we can assign all the p_next pointers, and know they will remain valid
//     /// until the [`Linked`] is dropped.
//     fn link<'a>(&'a self) -> Linked<'a, Self>;
// }

/// Represents a collection of structs that can be included in a pNext chain
///
/// This is used to extend functionality in Vulkan by appending
/// structs to a particular struct and forming a linked list.
pub unsafe trait Pnext<S, Dep> {
    type Output<Tag>: LinkMut<S>;
    #[doc(hidden)]
    fn p_next_uninit<Tag>() -> std::mem::MaybeUninit<Self::Output<Tag>> {
        std::mem::MaybeUninit::uninit()
    }
}

/// Link a collection of extension structs
///
/// ensure the sType is correctly set, and
/// set pNext to link each individual struct together.
#[doc(hidden)]
pub unsafe trait LinkMut<S> {
    fn link_mut(this: *mut Self) -> *mut S::Wrapped
    where
        S: Wrapper;
}

#[repr(transparent)]
#[doc(hidden)]
/// Created as a pointer for a p_next chain that is properly linked
pub struct Linked<'a, T: ?Sized>(*const BaseInStructure, PhantomData<&'a T>);

impl<'a, T: ?Sized> Linked<'a, T> {
    pub unsafe fn new(ptr: *const BaseInStructure) -> Self {
        Self(ptr, PhantomData)
    }
}

unsafe impl<T> ConvertWrapper<*const BaseInStructure> for Linked<'_, T> {}

#[repr(transparent)]
#[doc(hidden)]
/// Created as a pointer for a mutable p_next chain that is properly linked
pub struct LinkedMut<'a, T: ?Sized>(*mut BaseOutStructure, PhantomData<&'a mut T>);

impl<'a, T: ?Sized> LinkedMut<'a, T> {
    pub unsafe fn new(ptr: *mut BaseOutStructure) -> Self {
        Self(ptr, PhantomData)
    }
}

unsafe impl<T> ConvertWrapper<*mut BaseOutStructure> for LinkedMut<'_, T> {}

#[doc(hidden)]
pub mod p_next_macro_prelude {
    pub use super::LinkMut;
    pub use crate::type_conversions::{ConvertWrapper, Wrapper};
    pub use vk_safe_sys::structs as vk_structs;
    pub use vk_safe_sys::StructExtends;
    pub use vk_safe_sys::{BaseOutStructure, BaseStructure, BaseStructureMut};
}

/// Make a list of types to be included in a p_next chain
///
/// When making a p_next chain for the Vulkan implementation to output data to,
/// list the types of the extension structs that you would like to include. The
/// underlying api will create the p_next chain and return the base main structure
/// along with all members in the p_next chain.
#[macro_export]
macro_rules! p_next {
    // When using p_next in the case of retrieving data output by the implementation,
    ( $($name:ident),+ $(,)? ) => {{
        $crate::p_next_inner!(StructExtension, Pnext: $($name),* );
        Pnext
    }};
}

#[macro_export]
macro_rules! p_next_inner {
    ( $s_name:ident , $p_name:ident : $($name:ident),+ $(,)? ) => {
        use $crate::struct_extension::p_next_macro_prelude::*;
        $( use vk_structs::{$name}; )*

        // It is assumed that all BaseStructureMut types are only used to return
        // information from the Vulkan implementation, which does not need tyo be
        // generic over anything other than the tag. Thus, we just assume that
        // our Pnext collection only needs to be generic over the tag.

        /// This is a collection of extension structs containing:
        $( #[doc = stringify!($name)] )*
        #[doc = concat!("Defined at ", file!(), line!())]
        #[allow(non_snake_case)]
        pub struct $s_name<S, Tag> {
            pub base: S,
            $(pub $name: $crate::vk::$name<Tag>,)*
            _phantom: std::marker::PhantomData<Tag>,
        }

        unsafe impl<S: Wrapper, Tag> LinkMut<S> for $s_name<S, Tag> where S::Wrapped: BaseStructureMut {
            fn link_mut(this: *mut Self) -> *mut S::Wrapped
            where
                S: Wrapper
            {
                let mut ptr: *mut BaseOutStructure = std::ptr::null_mut();
                $(
                    let s: *mut $name = unsafe { (&raw mut (*this).$name) }.to_c();
                    BaseStructure::set_s_type(s);
                    BaseStructureMut::p_next_mut(s, ptr);
                    ptr = s.cast();
                )*
                let base: *mut S::Wrapped = unsafe { (&raw mut (*this).base) }.to_c();
                BaseStructure::set_s_type(base);
                BaseStructureMut::p_next_mut(base, ptr);
                base
            }
        }

        impl<S: std::fmt::Debug, Tag> std::fmt::Debug for $s_name<S, Tag> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("")
                    .field(&self.base)
                    $( .field(&self.$name) )*
                    .finish()
            }
        }

        struct $p_name;

        unsafe impl<S: Wrapper<Wrapped: BaseStructureMut>, Dep> $crate::struct_extension::Pnext<S, Dep> for $p_name
        where
            $(
                $name: StructExtends<S::Wrapped>,
                Dep: vk_structs::struct_dependencies::$name::HasDependency,
            )*
        {
            type Output<Tag> = $s_name<S, Tag>;
        }

        $p_name
    };
}
