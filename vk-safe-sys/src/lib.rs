#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

#![recursion_limit = "1000"]

// trace_macros!(true);

#[macro_use]
pub mod generated; // TODO I do not think I want this public, but need type defs for now

#[macro_use]
mod utils;
mod definitions;

#[macro_use]
pub mod commands; // TODO I do not think I want this public, but I need the fptr traits for now
pub mod version;
pub mod extension;

pub use generated::*;

pub use commands::LoadCommands;

// include!{concat!(env!("OUT_DIR"), "/vk.rs")}