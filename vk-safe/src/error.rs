use vk_safe_sys as vk;

use std::fmt::{Debug, Display};

/// custom error type for this crate
/// may consider using the 'anyhow' crate at some point, but I want to keep dependencies at minimum for now
///
/// The main purpose of this is just to ensure everything reports errors consistently and provide one place to make changes
#[derive(Debug)]
pub struct Error(Box<dyn std::error::Error + 'static>);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl<E: std::error::Error + 'static> From<E> for Error {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

impl From<Error> for Box<dyn std::error::Error> {
    fn from(e: Error) -> Self {
        e.0
    }
}

/// Some Vulkan commands return VkResult, some return nothing.
/// This trait allows handling each case in the same way
pub(crate) trait VkResultExt {
    fn is_err(&self) -> bool;
    #[allow(unused)]
    fn is_success(&self) -> bool;
    fn get_error(self) -> vk::Result;
}

impl VkResultExt for vk::Result {
    fn is_err(&self) -> bool {
        self.is_err()
    }
    fn is_success(&self) -> bool {
        self.is_success()
    }
    fn get_error(self) -> vk::Result {
        self
    }
}

impl VkResultExt for () {
    fn is_err(&self) -> bool {
        false
    }

    fn is_success(&self) -> bool {
        true
    }

    fn get_error(self) -> vk::Result {
        panic!("cannot get error: this should be an infallible case")
    }
}

/// Need to do C style error checking when calling the raw vulkan function pointers
macro_rules! check_raw_err {
    ( $result:ident ) => {
        #[allow(unused_imports)]
        use $crate::error::VkResultExt;
        if $result.is_err() {
            Err($result.get_error())?
        }
    };
}
