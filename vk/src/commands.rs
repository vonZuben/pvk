use std::os::raw::{c_void, c_char};
use crate::utils::{Hnode, End};

pub trait FunctionPointer: Copy {
    const VK_NAME: *const c_char;
    type Fptr;
    unsafe fn new(ptr: *const c_void) -> Self;
    fn fptr(&self) -> Self::Fptr;
}

#[derive(Copy, Clone, Hash)]
pub struct Loader<Cmd>(Cmd);

pub trait FunctionLoader: Fn(*const c_char) -> crate::generated::definitions::PFN_vkVoidFunction + Copy {}
impl<F> FunctionLoader for F where F: Fn(*const c_char) -> crate::generated::definitions::PFN_vkVoidFunction + Copy {}

impl<Cmd: FunctionPointer> Loader<Cmd> {
    fn load(f: impl FunctionLoader) -> Result<Self, &'static str> {
        let fptr = f(Cmd::VK_NAME);
        if fptr.is_null() {
            return Err("can't load fn"); // TODO should make better error type
        }
        else {
            unsafe {
                Ok( Self(Cmd::new(fptr)) )
            }
        }
    }
    pub fn fptr(&self) -> Cmd::Fptr {
        self.0.fptr()
    }
}

pub trait LoadCommands : Sized {
    fn load(f: impl FunctionLoader) -> Result<Self, &'static str>;
}

impl LoadCommands for End {
    fn load(f: impl FunctionLoader) -> Result<Self, &'static str> {
        Ok(Self)
    }
}

impl<Cmd: FunctionPointer, Tail> LoadCommands for Hnode<Loader<Cmd>, Tail>
where
    Tail: LoadCommands,
{
    fn load(f: impl FunctionLoader) -> Result<Self, &'static str> {
        Ok(Self {
            head: Loader::<Cmd>::load(f)?,
            tail: Tail::load(f)?,
        })
    }
}

macro_rules! make_fptr_traits {
    ( $($name:ident ),* $(,)? ) => {
        $(
            pub trait $name {
                fn fptr(&self) -> $crate::generated::command_function_pointers::$name;
            }

            impl FunctionPointer for $crate::commands::function_pointer_wrappers::$name {
                const VK_NAME: *const c_char = unsafe { std::mem::transmute(concat!(stringify!($name), "/0").as_ptr()) };
                type Fptr = $crate::generated::command_function_pointers::$name;
                unsafe fn new(ptr: *const c_void) -> Self {
                    Self(::std::mem::transmute(ptr))
                }
                fn fptr(&self) -> Self::Fptr {
                    self.0
                }
            }
        )*
    };
}

macro_rules! make_fptr_wrappers {
    ( $($name:ident),* $(,)? ) => {
        $(
            #[repr(transparent)]
            #[derive(Copy, Clone)]
            pub struct $name(pub(crate) $crate::generated::command_function_pointers::$name);
        )*
    };
}

macro_rules! make_loaders {
    ( $($name:ident),* $(,)? ) => {
        $(
            pub type $name = Loader<function_pointer_wrappers::$name>;
        )*
    };
}

// TODO this is related to extensions and should probably be in a different file
macro_rules! make_extention_implementor {
    ( $m_name:ident => $($ex:ident),* ) => {
        #[macro_export]
        macro_rules! $m_name {
            ( $name:ident ) => {
                $crate::impl_fptr_traits!($name => $($ex),*);
            };
        }
    };
}

#[macro_export]
macro_rules! impl_fptr_traits {
    ( $name:ident => $($command:ident),* ) => {
        $(
            impl $crate::$command for $name {
                fn fptr(&self) -> $crate::generated::function_pointers::$command {
                    use $crate::utils::Get;
                    let loader: &$crate::commands::loaders::$command = self.get();
                    loader.fptr()
                }
            }
        )*
    };
}

// #[macro_export]
// TODO this is related to features (e.g. VULKAN_1_0) and should probably be in a different file
macro_rules! make_commands_type {
    ( $name:ident => $($command:ident),* ) => {
        pub type $name = hlist_ty!( $($crate::command::$command),* );
        $crate::impl_fptr_traits!($name => $($command),*);
    };
}

use_command_function_pointer_names!(make_fptr_traits);

pub mod function_pointer_wrappers {
    use super::*;
    use_command_function_pointer_names!(make_fptr_wrappers);
}

pub mod loaders {
    use super::*;
    use_command_function_pointer_names!(make_loaders);
}