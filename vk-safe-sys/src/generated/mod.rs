// #[cfg(not(feature = "generate"))]
// #[macro_use]
// mod pre_built;

// TODO currently there is no pre_built code, so should fix this in future
// #[cfg(not(feature = "generate"))]
// pub use pre_built::*;

#[cfg(feature = "generate")]
include! {concat!(env!("OUT_DIR"), "/lib.rs")}

impl Result {
    pub fn is_err(&self) -> bool {
        self.0 < 0
    }
    pub fn is_success(&self) -> bool {
        self.0 >= 0
    }
}

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for Result {}
