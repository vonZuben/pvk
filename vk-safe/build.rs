use vk_safe_sys::vuid_check;

use std::env::var_os;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "check_vuids")]
fn check_vuids() -> Result<()> {
    let manifest_dir: PathBuf = var_os("CARGO_MANIFEST_DIR")
        .ok_or("ERROR: cannot get CARGO_MANIFEST_DIR")?
        .into();
    let manifest_dir = manifest_dir.join("src");

    vuid_check::check_vuids(manifest_dir.as_path())?;

    Ok(())
}

fn main() -> Result<()> {
    #[cfg(feature = "check_vuids")]
    check_vuids()?;

    Err("check")?;
    Ok(())
}
