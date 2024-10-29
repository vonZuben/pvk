use generator::sdk;
use std::path::{Path, PathBuf};

use std::process::Command;

fn main() {
    let children = [
        compile_glsl_resource("shader.vert", "vertex.spv"),
        compile_glsl_resource("shader.frag", "fragment.spv"),
    ];

    for (mut child, src_file) in children {
        child
            .wait()
            .expect(format!("error while compiling {}", src_file).as_str());
    }
}

/// compile and register glsl source file
///
/// Compile `src_file_name` with glslang provided by VULKAN_SDK, and
/// out put the compiled code to `out_file_name`.
/// Set `cargo::rerun-if-changed={src_file_name}` to automatically
/// re compile the glsl when changed.
fn compile_glsl_resource<'s>(
    src_file_name: &'s str,
    out_file_name: &str,
) -> (std::process::Child, &'s str) {
    let glslang_path = sdk::sdk_bin_path()
        .expect("can't find Vulkan SDK binaries path")
        .join("glslang");

    println!("glslang: {}", glslang_path.display());

    let project_folder = env!("CARGO_MANIFEST_DIR");
    let mut shader_src = PathBuf::new();
    shader_src.extend([project_folder, "resources", src_file_name]);

    println!("cargo::rerun-if-changed={}", shader_src.display());

    let out_dir = std::env::var("OUT_DIR").expect("Could not get OUT_DIR");
    let shader_out = Path::new(&out_dir).join(out_file_name);

    (
        Command::new(&glslang_path)
            .arg("-V")
            .arg(shader_src)
            .arg("-o")
            .arg(shader_out)
            .spawn()
            .expect("Could not run glslang"),
        src_file_name,
    )
}
