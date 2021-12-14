#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

#![recursion_limit = "1000"]

#[macro_use]
mod generated;

#[macro_use]
mod utils;
mod definitions;

#[macro_use]
mod commands;
pub mod version;
pub mod extension;

pub use generated::*;

pub use commands::LoadCommands;

// include!{concat!(env!("OUT_DIR"), "/vk.rs")}