/*! vk-safe provides a safe low level rust api for vulkan

# Getting started

This API is meant to be one-to-one with the C Vulkan API, as much as possible. Exceptions to this rule should be documented on a case by case basis.
Thus, getting started with this API is very similar to getting started with Vulkan in C. There are many resources online, but a good start would be [Vulkan tutorial](https://vulkan-tutorial.com/).

### Example (bare minimum to get a Device context)
```
use vk_safe as vk;

// declare the Vulkan version we are targeting
// must satisfy Device version <= Instance version
// (verified when used together)
vk::instance_context!(InstanceContext: VERSION_1_0);
vk::device_context!(DeviceContext: VERSION_1_0);

// configure and create instance
let app_info = vk::ApplicationInfo::new(InstanceContext);
let instance_info = vk::InstanceCreateInfo::new(&app_info);
let instance = vk::create_instance(&instance_info).unwrap();

// create a scope in which to use the instance (See Scope documentation below)
vk::scope(instance, |instance| {

    // get physical devices
    let physical_devices = instance
        .enumerate_physical_devices(Vec::new())
        .unwrap();

    for physical_device in physical_devices.iter() {
        vk::scope(physical_device, |physical_device| {
            // discover queues on the physical device
            let queue_family_properties = physical_device
                .get_physical_device_queue_family_properties(Vec::new())
                .unwrap();

            // configure queues that support graphics
            queue_family_properties.config_scope(|qp| {
                let mut queue_configs = vec![];
                let priorities = vk::QueuePriorities::new(&[1.0; 10]);
                for p in qp {
                    if p.queue_flags.contains(vk::QueueFlags::GRAPHICS_BIT) {
                        queue_configs.push(
                            vk::DeviceQueueCreateInfo::new(priorities.with_num_queues(p.queue_count), p)
                                .unwrap(),
                        )
                    }
                }

                // configure and create device
                let device_create_info = vk::DeviceCreateInfo::new(DeviceContext, &queue_configs);
                let device = physical_device
                    .create_device(&device_create_info, &queue_family_properties)
                    .unwrap();
            });
        })();
    }
})();

```

# About the documentation

Since this is generally mean to be as close to the c Vulkan API as possible, most methods do not have documentation here, and the official documentation should be checked,
Links to the official documentation should be provided here per method.

# Why I made this

The intention is to provide an API that is as close as possible to the c vulkan api, while also being safe to use.
I wanted to make this for the following reasons:
1. to become better with Rust and Vulkan
2. to create an API that provides a combination of "low level" and "safety" that I could not find with other Rust Vulkan APIs
3. to create a safe Vulkan API that is as zero overhead as possible
(In particular, I am interested in having access to low level memory APIs in Vulkan, while also having safety, for implementing a Wayland compositor)
4. to **experiment** with how far I can push the type system to create a zero overhead safe Vulkan API

## Comparison to other APIs
- [Ash](https://github.com/ash-rs/ash) is a true low level Vulkan API for rust where everything is safe. (I want to build in safety)
- [Vulkano](https://github.com/vulkano-rs/vulkano) calls itself a "low-levelish API" and a "High-level Rust API" (so I'll say Medium level lol), and it provides safety.
Vulkano is far from zero overhead due to using thing like Arc, and performing potentially heavy verification for all api calls.
(normally people using the c API would only use validation layers such as VK_LAYER_KHRONOS_validation during development, and then remove all validation in a release build, but
I am not sure this can be done with Vulkano at this time)
- [Wgpu](https://github.com/gfx-rs/wgpu) is a high level API that works on top of Vulkan, and other graphics APIs. Since it is not exclusive to Vulkan, it cannot provide the low level APIs I want.

*/

#[macro_use]
mod error;

#[macro_use]
mod helper_macros;
#[macro_use]
mod error_macros;

mod array_storage;
mod flags;
mod type_conversions;
mod vk_str;

mod device_type;
mod entry; // not finalized on if this should be pub
mod instance_type;
mod physical_device;
mod queue_type;
mod scope;

pub use vk_safe_sys::VkVersion;
pub use vk_str::VkStr;

pub use vk_safe_sys::generated_vulkan::bitmasks::*;
pub use vk_safe_sys::generated_vulkan::enum_variants::*;
pub use vk_safe_sys::generated_vulkan::enumerations::*;
pub use vk_safe_sys::{device_context, instance_context};

pub use device_type::device_exports::*;
pub use entry::*;
pub use instance_type::instance_exports::*;
pub use physical_device::physical_device_exports::*;

pub use scope::scope;

pub use flags::*;

pub mod instance {
    pub use vk_safe_sys::extension::instance::traits::*;
    pub use vk_safe_sys::version::instance::traits::*;
}

pub mod device {
    pub use vk_safe_sys::extension::device::traits::*;
    pub use vk_safe_sys::version::device::traits::*;
}
