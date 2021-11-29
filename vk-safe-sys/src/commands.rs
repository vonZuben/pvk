use crate::utils::{End, Hnode};
use std::os::raw::{c_char, c_void};

pub trait FunctionPointer: Copy {
    const VK_NAME: *const c_char;
    type Fptr: Sized;
    unsafe fn new(ptr: *const Self::Fptr) -> Self;
    fn fptr(&self) -> Self::Fptr;
}

#[derive(Copy, Clone, Hash)]
pub struct Loader<Cmd>(Cmd);

pub type VkVoidFunction = crate::generated::PFN_vkVoidFunction;

pub trait FunctionLoader:
    Fn(*const c_char) -> Option<VkVoidFunction> + Copy
{
}
impl<F> FunctionLoader for F where
    F: Fn(*const c_char) -> Option<VkVoidFunction> + Copy
{
}

impl<Cmd: FunctionPointer> Loader<Cmd> {
    fn load(f: impl FunctionLoader) -> Result<Self, &'static str> {
        let fptr = f(Cmd::VK_NAME);
        match fptr {
            Some(fptr) => Ok(Self(unsafe {
                Cmd::new((&fptr as *const VkVoidFunction).cast())
            })),
            None => Err("can't load fn"),
        }
    }
    pub fn fptr(&self) -> Cmd::Fptr {
        self.0.fptr()
    }
}

pub trait LoadCommands: Sized {
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
    ( $($name:ident -> $string:expr);* $(;)? ) => {
        $(
            pub trait $name {
                fn fptr(&self) -> $crate::generated::$name;
            }

            impl FunctionPointer for $crate::commands::function_pointer_wrappers::$name {
                const VK_NAME: *const c_char = concat!($string, "/0").as_ptr().cast();
                type Fptr = $crate::generated::$name;
                unsafe fn new(fptr: *const Self::Fptr) -> Self {
                    Self(*fptr)
                }
                fn fptr(&self) -> Self::Fptr {
                    self.0
                }
            }
        )*
    };
}

macro_rules! make_fptr_wrappers {
    ( $($name:ident -> $string:expr);* $(;)? ) => {
        $(
            #[repr(transparent)]
            #[derive(Copy, Clone)]
            pub struct $name(pub(crate) $crate::generated::$name);
        )*
    };
}

macro_rules! make_loaders {
    ( $($name:ident -> $string:expr);* $(;)? ) => {
        $(
            pub type $name = Loader<function_pointer_wrappers::$name>;
        )*
    };
}

#[macro_export]
macro_rules! impl_fptr_traits {
    ( $name:ident => $($command:ident),* ) => {
        $(
            impl $crate::commands::$command for $name {
                fn fptr(&self) -> $crate::generated::$command {
                    use $crate::utils::Get;
                    let loader: &$crate::commands::loaders::$command = self.get();
                    loader.fptr()
                }
            }
        )*
    };
}

macro_rules! make_trait_aliases {
    ( $( $name:ident = $alias:ident ),* ) => {
        $(
            pub trait $name : $alias {}
            impl<T> $name for T where T: $alias {}
        )*
    }
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