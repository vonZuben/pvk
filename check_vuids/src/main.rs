use std::env::args_os;
use std::path::PathBuf;

use vk_safe_sys::validation;

mod parse;
mod vuid_check;
mod vuids;

/**
Check VUIDs in all files in a given directory
 */
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let check_dir: PathBuf = args_os()
        .last()
        .ok_or("USAGE: provide path to directory to check")?
        .into();

    let vuids = validation::get_vuids();

    if !check_dir.is_dir() {
        Err("USAGE: provide path to directory to check")?
    } else {
        vuid_check::check_vuids(check_dir.as_path())
    }
}
