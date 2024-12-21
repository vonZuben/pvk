# PVK

This is a workspace containing Rust crates which provide a Vulkan API for Rust that aims to be safe with as little overhead as possible.

# 📋 List of crates

- `vk-safe` is the main crate which provides the safe Vulkan API
- `vk-safe-demo` is a demonstration of vk-safe
- `vk-safe-sys` contains generated code for directly calling the C Vulkan API
- `generator` is used while compiling vk-safe-sys to generate the C Vulkan API
- `check-vuids` is a tool used to ensure that all Vulkan valid usage rules are verified for each item in the Vulkan API.
- `krs-quote` is a [`quote`](https://docs.rs/quote/latest/quote/) *like* crate used in the generator

#### NOTES
- *there are a couple other old crates in this workspace that are from old experiments and are likely to be removed*
- *all crate names are temporary and should be changed before making published versions*

# 🛠️ Build

Minimum Supported Rust Version: 1.82.0 (using opaque type precise capturing (Rust RFC 3617))

To build and run `vk-safe-demo`:
1. Install the [VulkanSDK](https://vulkan.lunarg.com/sdk/home) for your system.
2. Set the `VULKAN_SDK` or `VK_SDK_PATH` environment variables. e.g. `VULKAN_SDK=.../1.3.268.0/x86_64`
    - On Windows, the environment variables should be set by the VulkanSDK installer.
    - On linux, the VulkanSDK provides a `setup-env.sh` which can set the environment variables.
    (If you are using an IDE, you may need to set the environment variables in your projects configuration)
3. Use `cargo run --bin vk-safe-demo` in your terminal from the workspace root. This should build and run a simple
example program that makes use of all the currently implemented Vulkan APIs.

⚠️ Mac is not tested. Mac does not natively support Vulkan and it is necessary to use MoltenVK.
I do not have access to a Mac computer and have not tested anything in this regard.

#### Build details

The main `vk-safe` crate depends on `vk-safe-sys` and `check-vuids`, which each in turn depend on the `generator`.

`vk-safe-sys` and `check-vuids` have `build.rs` scripts for running the generator to generate the the C Vulkan API
and a list of valid usage rules respectively. The generated code depends on the `vk.xml` and `validusage.json` files that are found
in the installed VulkanSDK.

`vk-safe-sys` also links to the systems Vulkan library (e.g. libvulkan.so on Linux). If your computer can run Vulkan programs, you
should already have this. However, it is also provided in the VulkanSDK.

💁 It is possible to build `vk-safe` without the entire VulkanSDK installed. In this case, you will need to obtain valid copies
of `vk.xml` and `validusage.json`, such as from
[Vulkan-Headers GitHub](https://github.com/KhronosGroup/Vulkan-Headers/tree/main/registry)
and put them in a folder of your choice. Then you need to set the following environment variables (you can change the file
names too, but they need to be specified):
- `VK_XML_OVERRIDE=**my-path**/vk.xml`
- `VALIDUSAGE_JSON_OVERRIDE=**my-path**/validusage.json`

`vk-safe-demo` requires `glslang` from the VulkanSDK bin tools in order to automatically compile shaders.
- `VK_BIN_OVERRIDE=**bin bath**` can be set to a path containing glslang if the whole VulkanSDK is not installed

## ⚠️ This is still very much a work in progress

This is VERY early in development and there are no published crates yet. Most of the interesting documentation is in rustdoc
style comments and it is necessary to download this workspace and generate it with `cargo doc` (in which case you also need to be
able to build the code as per the above details).