/*!
Check source files for check_vuids!(target) macro calls.

for each such macro call found, check if the VUIDs in the same block are up to date
and update as necessary

With no arguments passed, it will automatically search all source files in the workspace vk-safe/src

One argument for the desired search directory can be provided and searched instead

# !!NOTE
Do NOT tamper with the bodies of the generated version! and description! macro calls. You can
(and should) delete the generated old_description! macro call after you are done checking the updated
VUID.
 */
use std::env::{args_os, current_exe};
use std::path::PathBuf;

mod file_edits;
mod must_next;
mod parse;
mod vuid_check;
mod vuids;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const USAGE_ERROR: &'static str =
    "USAGE: run from workspace root or provide path to directory to check (note: expects exactly zero on one arguments)";

fn check_in_workspace() -> Option<PathBuf> {
    let check_dir = PathBuf::from("vk-safe/src");

    if check_dir.exists() {
        Some(check_dir)
    } else {
        None
    }
}

fn main() -> Result<()> {
    let check_dir = match args_os().last() {
        Some(arg) => {
            if arg == current_exe()? {
                check_in_workspace().ok_or(USAGE_ERROR)?
            } else {
                arg.into()
            }
        }
        None => check_in_workspace().ok_or(USAGE_ERROR)?,
    };

    if !check_dir.is_dir() {
        Err(USAGE_ERROR)?
    } else {
        let vuid_collection = vuids::VuidCollection::new()?;
        vuid_check::check_vuids(check_dir.as_path(), &vuid_collection)
    }
}
