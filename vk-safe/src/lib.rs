/*! vk-safe provides a safe low level Rust API for vulkan

## ⚠️ This is still very much a work in progress
This library is going thorough a lot of experimentation to see how far the type system can be pushed to provide a zero-overhead safe API for Vulkan.
At this stage, only a few Vulkan APIs are implemented. However, this should be a good representation of what the overall API will look like when done.
i.e. the "tricks" currently used with the type system to make the API safe should currently be pretty representative of the final product, and from now,
simply a lot of work needs to be put into implementing all the actual Vulkan APIs.

# Getting started

At the outset, this API is meant for people who know how to use Vulkan, or maybe for those who want to learn Vulkan.
This API is meant to be one-to-one with the C Vulkan API, as much as possible. Exceptions to this rule should be documented on a case by case basis.
Getting started with this API is very similar to getting started with Vulkan in C. There are many resources online, but a good start would be [Vulkan tutorial](https://vulkan-tutorial.com/).

In view of the above, it is best to use this API while first understanding the C Vulkan API, and then the differences in vk-safe to make it more Rusty.
When you are ready, take a look at [`create_instance()`].

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

# Key Differences from C Vulkan API

#### Naming convention
Vulkan items (commands, structs, etc.) are renamed in vk-safe to follow Rust naming conventions.
Names from Vulkan are converted by cutting off the leading "Vk" or "vk", and then converting the remaining name in-line with
[RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md)

#### Commands are methods
Most Vulkan commands take a dispatchable handle as the first argument. In vk-safe, the dispatchable handle has methods corresponding to these commands, and the first parameter is
set to the corresponding handle automatically. Few commands, such as [`create_instance()`] take no dispatchable handle, and are plain functions.

#### Dispatchable handle methods require 'Scope'
vk-safe uses an invariant lifetime trick to "tag" instances of dispatchable handles within a "Scope". Resources which are later obtained from the dispatchable
handle have the same "tag". This ensures that resources can only be used with the corresponding dispatchable handle from which they were created.

In Rust today, it is only possible to make a 'Scope' at a function boundary. Thus, each handle you want to use needs to be passed into it's own function with [`scope()`], which creates
closures, that can be considered as individual units of execution, and the user can decide how to handle them. One consequence of this is that in order to handle many different
handles at the same time, it is necessary to make them into sub-scopes (closures within closures), or run the units of execution concurrently such as with threads or async Rust. Sub-scopes are
tightly bound to the structure of your code and are only a good choice when you already know how many handles you are using. For the case of handling dynamic numbers of handles,
such as PhysicalDevice's, it is better to run concurrent units of execution (of course each handle could be used sequentially in a loop, but usually you want to use all the available
PhysicalDevice's for the whole program runtime concurrently).

Please also see the [`Scope`] and [`RefScope`] types for implementation details.

A consequence of making handles only safe to use in scopes is that the methods of handles are implemented through [`RefScope`].

ℹ️ I recently found [generativity crate](https://docs.rs/generativity/latest/generativity/), and I am investigating if it is sound, since it would be easier to use.
In any event, it should still be necessary to make concurrent units of execution in order to use multiple handles at the same time.

#### Trait representations of handles
Due to the above mentioned "tag" and "Scope" trick, the concrete types of handles are complex, since all handles are generic over their "tag". To alleviate this,
the main way of specifying handle types is to do so generically with traits. Somewhat contrary to the above 'Naming convention' rule, the concrete type of a
handle is `HandleNameType` (with 'Type' appended). There is then a corresponding `HandleName` trait which should be the main way of specifying the types you are using.

The `HandleName` trait is normally implemented for [`Scope<'_, HandleNameType>`](Scope). Some handles do not need scopes, and `HandleName` trait is implemented directly for `HandleNameType`.

#### Returning Result
All Vulkan commands that can fail will return a Result. There Err variant is currently a placeholder dyn Error type. This should be changed in future to an Error type
that enables handling specific Vulkan errors more easily.

#### Structs are read-only by default
Most structs have a thin wrapper with a Deref implementation to provide read-only access. Some structs have more specific methods to provide safe access.
Vulkan has many "Info" structs that the user creates and passes to commands, which have appropriate constructor methods to ensure valid usage. Some structs may have other
methods for safely enabling specific use cases.

#### Enumerator commands use ArrayStorage
Vulkan has many "Enumerate" or "Get" commands which take a pointer / length for an array, to which return data will be written. Said commands can also be used to query length
of data to be returned by passing a null pointer. **In vk-safe**, "Enumerate" or "Get" commands take a storage type which implements the [`ArrayStorage`] trait.

#### Enumerations and BitFlags are structs with associated constants
This allows unknown variants or bits to be explicitly handled. This is necessary because a Vulkan implementation working on a higher version than this library was generated with can lead to obtaining
Unknown variants or bits. Since associated constants cannot currently be imported into the namespace with `use`, vk-safe also provides modules corresponding to each
Enumeration and BitFlag type, with all variants or bits as plain constants which can be imported. The type and module names follow expected Rust convention.

```
# mod example {
// illustrative of actual implementation
pub struct StencilOp(pub(crate) i32);

impl StencilOp {
    pub const KEEP: Self = Self(0);
    pub const ZERO: Self = Self(1);
    pub const REPLACE: Self = Self(2);
    // and more
}

pub mod stencil_op {
    use super::StencilOp;
    pub const KEEP: StencilOp = StencilOp::KEEP;
    pub const ZERO: StencilOp = StencilOp::ZERO;
    pub const REPLACE: StencilOp = StencilOp::REPLACE;
    // and more
}
# }
```

#### 🚧 AllocationCallbacks
Vulkan supports AllocationCallbacks mostly for debugging purposes. These are not currently supported in vk-safe. Adding them will *most likely* be a breaking change, assuming that
parameters will be added to the respective `create_*` commands. (e.g. [`create_instance()`] will likely have an allocation_callbacks parameter added).

## VUIDs (implementation detail)
All Vulkan APIs have valid usage rules that must be followed. Each valid usage rule has a VUID (Valid Usage Identifier). For all v-safe APIs,
the relevant VUIDs are *manually* checked against the Vulkan documentation. To help ensure that all VUIDs are checked and updated with changes
to Vulkan, a [check_vuids] tool is provided for development purposes to help automatically include the VUID rules in the source code.

After the rules are included in the source code, vk-safe attempts to ensure all ensure the rules are followed by designing the APIs to make use
of the type system as much as reasonably possible.

Some things are too complex to express in the type system (or it could be expressed but be too hard to use), so `const` computation is used when
possible (a big example of this is with APIs that take image format, tiling, type, and related flags etc., which have many possible combinations
and only a subset of valid / sensible combinations). It is recommended to make use of the `const` computation when possible to reduce runtime checks,
and also get earlier compile time errors.

Lastly, some things must be checked at runtime with regard to information that must be queried from the system. As much as possible, the API is
designed to make the user perform the checks automatically simply by using the API normally, and the type system is used to keep relevant information.
This, way there should be no *real* overhead since the user should need to do these checks anyway when initially obtaining like memory properties,
and then API's can rely on the information stored in the types.

# 🤔 Why I made this

The intention is to provide an API that is as close as possible to the C Vulkan API, while also being safe to use.
I wanted to make this for the following reasons:
1. to become better with Rust and Vulkan
2. to create an API that provides a combination of "low level" and "safety" that I could not find with other Rust Vulkan APIs
3. to create a safe Vulkan API that is as zero overhead as possible
4. to **experiment** with how far I can push Rust and the type system to create a zero overhead safe Vulkan API
5. to code as a hobby (I can only spend my own limited free time on this and progress is very slow)

## Other APIs
- [Ash](https://github.com/ash-rs/ash) is a true low level Vulkan API for Rust where everything is unsafe. **Ash is actually a big inspiration for features vk-safe, other than safety**
- [Vulkano](https://github.com/vulkano-rs/vulkano) calls itself a "low-levelish API" and a "High-level Rust API" in the README. From my own review, it seems pretty low level. However,
Vulkano is far from zero overhead due to using thing like Arc, and performing potentially heavy verification for all API calls.
(normally people using the c API would use validation layers such as VK_LAYER_KHRONOS_validation during development, and then remove all validation in a release build, but
I am not sure this can be done with Vulkano since validation is built in as runtime checks). I want vk-safe to represent validation in the type system so there is no overhead.
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

pub use array_storage::ArrayStorage;

pub use vk_safe_sys::generated_vulkan::bitmasks::*;
pub use vk_safe_sys::generated_vulkan::enum_variants::*;
pub use vk_safe_sys::generated_vulkan::enumerations::*;
pub use vk_safe_sys::{device_context, instance_context};

pub use device_type::device_exports::*;
pub use entry::*;
pub use instance_type::instance_exports::*;
pub use physical_device::physical_device_exports::*;
pub use queue_type::queue_exports::*;

pub use scope::{scope, RefScope, Scope};

pub use flags::*;

pub mod instance {
    pub use vk_safe_sys::extension::instance::traits::*;
    pub use vk_safe_sys::version::instance::traits::*;
}

pub mod device {
    pub use vk_safe_sys::extension::device::traits::*;
    pub use vk_safe_sys::version::device::traits::*;
}
