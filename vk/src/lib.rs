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

mod utils;
mod definitions;
mod commands;


// include!{concat!(env!("OUT_DIR"), "/vk.rs")}