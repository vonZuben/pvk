#[cfg(not(feature = "generate"))]
#[macro_use]
mod pre_built;

#[cfg(not(feature = "generate"))]
pub use pre_built::*;

#[cfg(feature = "generate")]
#[macro_use]
mod generated {
    include!{concat!(env!("OUT_DIR"), "/vk.rs")}
}

#[cfg(feature = "generate")]
pub use generated::*;