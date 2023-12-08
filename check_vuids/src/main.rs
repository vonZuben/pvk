use std::env::args_os;
use std::path::PathBuf;

mod file_edits;
mod must_next;
mod parse;
mod vuid_check;
mod vuids;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/**
Check VUIDs in all files in a given directory
 */
fn main() -> Result<()> {
    let check_dir: PathBuf = args_os()
        .last()
        .ok_or("USAGE: provide path to directory to check")?
        .into();

    if !check_dir.is_dir() {
        Err("USAGE: provide path to directory to check")?
    } else {
        let vuid_collection = vuids::VuidCollection::new()?;
        vuid_check::check_vuids(check_dir.as_path(), &vuid_collection)
    }
}
