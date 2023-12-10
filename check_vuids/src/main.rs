use std::env::args_os;
use std::path::PathBuf;

mod file_edits;
mod must_next;
mod parse;
mod vuid_check;
mod vuids;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const USAGE_ERROR: &'static str =
    "USAGE: run from workspace root or provide path to directory to check";

/**
Check VUIDs in all files in a given directory
 */
fn main() -> Result<()> {
    let mut check_dir = PathBuf::from("vk-safe/src");

    if !check_dir.exists() {
        check_dir = args_os().last().ok_or(USAGE_ERROR)?.into();
    }

    if !check_dir.is_dir() {
        Err(USAGE_ERROR)?
    } else {
        let vuid_collection = vuids::VuidCollection::new()?;
        vuid_check::check_vuids(check_dir.as_path(), &vuid_collection)
    }
}
