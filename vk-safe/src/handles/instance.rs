use super::physical_device::PhysicalDeviceHandle;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::enumerator::Enumerator;
use crate::scope::{Captures, Tag};
use crate::VkVersion;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::has_command::DestroyInstance;
use vk::Version;

pub_use_modules!(
#[cfg(VK_VERSION_1_0)] {
    enumerate_physical_devices;
};
);

/// Main Vulkan object
///
/// [`Instance`] is the main object you create ([`create_instance`](crate::vk::create_instance))
/// in Vulkan that stores all application state. The primary thing you will want to do with
/// an Instance is enumerate the PhysicalDevices on the system ([`Instance::enumerate_physical_devices`])
///
/// Vulkan doc:
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkInstance.html>
pub trait Instance:
    DispatchableHandle<RawHandle = vk::Instance, Commands: vk::InstanceLabel> + ThreadSafeHandle
{
    const VERSION: VkVersion;

    #[cfg(VK_VERSION_1_0)]
    /// Enumerate PhysicalDevices on the system
    ///
    /// # Usage
    /// Use the resulting [`Enumerator`] to retrieve an array of [`PhysicalDeviceHandle`].
    /// Then you can iterate over the handles and tag each one that
    /// you want to use with a [`Tag`].
    ///
    /// # Example
    /// ```
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst(instance: impl Instance<Commands: vk::instance::VERSION_1_0>) {
    /// let physical_devices = instance
    ///     .enumerate_physical_devices()
    ///     .auto_get_enumerate()
    ///     .unwrap();
    ///
    /// for physical_device in physical_devices.iter() {
    ///     vk::tag!(tag);
    ///     let physical_device = physical_device.tag(&instance, tag);
    /// }
    /// # }
    /// ```
    ///
    /// Vulkan docs:
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html>
    fn enumerate_physical_devices<X>(&self) -> impl Enumerator<PhysicalDeviceHandle<Self>>
    where
        Self::Commands: vk::has_command::EnumeratePhysicalDevices<X>,
    {
        enumerate_physical_devices::enumerate_physical_devices(self)
    }
}

// Hidden type which implements [Instance]
struct _Instance<C: DestroyInstance<X>, X, T> {
    handle: vk::Instance,
    commands: C,
    tag: PhantomData<T>,
    destroy: PhantomData<X>,
}

unsafe impl<C: DestroyInstance<X>, X, T> Send for _Instance<C, X, T> {}
unsafe impl<C: DestroyInstance<X>, X, T> Sync for _Instance<C, X, T> {}
impl<C: DestroyInstance<X>, X, T> ThreadSafeHandle for _Instance<C, X, T> {}

impl<C: DestroyInstance<X>, X, T> fmt::Debug for _Instance<C, X, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance")
            .field("handle", &self.handle)
            // .field("version", &C::VERSION)
            .finish()
    }
}

impl<C: DestroyInstance<X>, X, T> _Instance<C, X, T> {
    fn new(handle: vk::Instance, commands: C, _tag: T) -> Self {
        Self {
            handle,
            commands,
            tag: PhantomData,
            destroy: PhantomData,
        }
    }
}

impl<C: DestroyInstance<X>, X, T> Handle for _Instance<C, X, T> {
    type RawHandle = vk::Instance;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<C: DestroyInstance<X>, X, T> DispatchableHandle for _Instance<C, X, T> {
    type Commands = C;

    fn commands(&self) -> &Self::Commands {
        &self.commands
    }
}

impl<C: DestroyInstance<X> + Version + vk::InstanceLabel, X, T> Instance for _Instance<C, X, T> {
    const VERSION: VkVersion = C::VERSION;
}

pub(crate) fn make_instance<C: DestroyInstance<X> + Version + vk::InstanceLabel, X>(
    handle: vk::Instance,
    commands: C,
    tag: Tag,
) -> impl Instance<Commands = C> + Captures<Tag> {
    _Instance::new(handle, commands, tag)
}

/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyInstance.html>
impl<C: DestroyInstance<X>, X, T> Drop for _Instance<C, X, T> {
    fn drop(&mut self) {
        check_vuids::check_vuids!(DestroyInstance);

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_00629: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "All child objects created using instance must have been destroyed prior to destroying"
            "instance"
            }

            // all child objects borrow the instance, and *normally* they are dropped/destroyed before the instance is destroyed
            // However, it is well known that rust does not guarantee that values will be dropped. Thus, we cannot enforce this rule
            // In any event, if a child object is not dropped (e.g. forgotten), it should never be used again or dropped. Thus, even if the Instance is
            // dropped, the child objects are merely leaked, and it is "assumed" that this is no real issue even in Vulkan.
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_00630: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkAllocationCallbacks were provided when instance was created, a compatible set"
            "of callbacks must be provided here"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_00631: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If no VkAllocationCallbacks were provided when instance was created, pAllocator must"
            "be NULL"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If instance is not NULL, instance must be a valid VkInstance handle"
            }

            // always a valid handle from creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        unsafe { self.commands.DestroyInstance().get_fptr()(self.handle, std::ptr::null()) }
    }
}
