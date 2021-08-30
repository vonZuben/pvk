#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

// trace_macros!(true);

#[cfg(not(generate))]
#[macro_use]
mod generated;

#[cfg(generate)]
#[macro_use]
#[path = concat!(env!("OUT_DIR"), "/mod.rs")]
mod generated;

#[macro_use]
mod utils;
mod definitions;
mod commands;
mod version;
mod extension;


// include!{concat!(env!("OUT_DIR"), "/vk.rs")}