# PVK

This workspace contains Rust crates which are all in support of making a low-level, safe, and zero overhead
(*as much as possible*) API for Vulkan.

# 📋 List of crates

- `vk-safe` is the main crate which provides the final API
- `vk-safe-sys` contains unsafe generated code for interfacing with Vulkan
- `generator` generates the unsafe code for vk-safe-sys
- `check-vuids` is a helper tool to ensure that all Vulkan valid usage rules are checked for each API item
- `krs-quote` is a [`quote`](https://docs.rs/quote/latest/quote/) *like* crate used in the generator

#### NOTES
- *there are a couple other old crates in this workspace that are from old experiments and are likely to be removed*
- *all crate names are temporary and should be changed before making published versions*

# 🛠️ Build

The recommended way to build is to install the [VulkanSDK](https://vulkan.lunarg.com/sdk/home) for your system,
and ensure that the `VULKAN_SDK` or `VK_SDK_PATH` environment variables are set. Then run `cargo run --example one`
from the workspace root. This should build and run a simple example program that makes use of all the currently implemented
Vulkan APIs.

- On Windows, the environment variables should already be set by the VulkanSDK installer.

- On linux, there should be a `setup-env.sh` in the VulkanSDK that can be used to set the environment. (If you are using an
IDE like VS code, you will probably need to set the environment variables in your projects configuration; see `setup-env.sh`
for details on what needs to be set; e.g. `VULKAN_SDK=.../1.3.268.0/x86_64`)

- Mac ⚠️ is not tested. Mac does not natively support Vulkan and it is necessary to use MoltenVK. I do not have access to
a Mac computer and have not tested anything in this regard.

#### Build details

The main `vk-safe` crate depends on `vk-safe-sys` and `check-vuids`, which each in turn depend on the `generator`.

`vk-safe-sys` and `check-vuids` have `build.rs` scripts for running the generator to generate the unsafe Vulkan interface
and list of valid usage rules respectively. The generated code depends on the `vk.xml` and `validusage.json` files that are found
in the installed VulkanSDK.

`vk-safe-sys` also links to the systems Vulkan library (e.g. libvulkan.so). If your computer can run Vulkan programs, you
should already have this. However, it is also provided in the VulkanSDK.

💁 It is possible to build everything in the workspace without the entire VulkanSDK installed. In this case, you will need to
obtain valid copies of `vk.xml` and `validusage.json`, such as from [Vulkan-Headers GitHub](https://github.com/KhronosGroup/Vulkan-Headers/tree/main/registry)
and put them in a folder of your choice. Then you need to set the following environment variables (you can change the file
names too, but they need to be specified):
- `VK_XML_OVERRIDE=**my-path**/vk.xml`
- `VALIDUSAGE_JSON_OVERRIDE=**my-path**/validusage.json`

## ⚠️ This is still very much a work in progress

This is VERY early in development and there are no published crates yet. Most of the interesting documentation is in rustdoc style comments
and it is necessary to download this workspace and generate it with cargo doc (in which case you also need to be able to build the code
as per the above details).