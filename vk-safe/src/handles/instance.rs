use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::scope::{Captures, Tag};
use crate::VkVersion;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::has_command::DestroyInstance;
use vk::Version;

handle_command_collection_trait!(
    NAME(InstanceCommands)
    IMPL( _Instance[C, X, T] where C: DestroyInstance<X> )
    #[cfg(VK_VERSION_1_0)] {
        EnumeratePhysicalDevices,
    }
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
    DispatchableHandle<RawHandle = vk::Instance, Commands: Version>
    + ThreadSafeHandle
    + InstanceCommands
{
    const VERSION: VkVersion;
}

// Hidden type which implements [Instance]
struct _Instance<C, X, T>
where
    C: DestroyInstance<X>,
{
    handle: vk::Instance,
    commands: C,
    tag: PhantomData<T>,
    destroy: PhantomData<X>,
}

// impl EnumeratePhysicalDevices

unsafe impl<C, X, T> Send for _Instance<C, X, T> where C: DestroyInstance<X> {}
unsafe impl<C, X, T> Sync for _Instance<C, X, T> where C: DestroyInstance<X> {}
impl<C, X, T> ThreadSafeHandle for _Instance<C, X, T> where C: DestroyInstance<X> {}

impl<C, X, T> fmt::Debug for _Instance<C, X, T>
where
    C: DestroyInstance<X>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance")
            .field("handle", &self.handle)
            // .field("version", &C::VERSION)
            .finish()
    }
}

impl<C, X, T> _Instance<C, X, T>
where
    C: DestroyInstance<X>,
{
    fn new(handle: vk::Instance, commands: C, _tag: T) -> Self {
        Self {
            handle,
            commands,
            tag: PhantomData,
            destroy: PhantomData,
        }
    }
}

impl<C, X, T> Handle for _Instance<C, X, T>
where
    C: DestroyInstance<X>,
{
    type RawHandle = vk::Instance;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<C, X, T> vk::Commands for _Instance<C, X, T>
where
    C: DestroyInstance<X>,
{
    type Commands = C;

    fn commands(&self) -> &Self::Commands {
        &self.commands
    }
}

impl<C, X, T> DispatchableHandle for _Instance<C, X, T> where C: DestroyInstance<X> {}

impl<C, X, T> Instance for _Instance<C, X, T>
where
    C: DestroyInstance<X> + Version,
{
    const VERSION: VkVersion = C::VERSION;
}

pub(crate) fn make_instance<C: DestroyInstance<X> + Version, X>(
    handle: vk::Instance,
    commands: C,
    tag: Tag,
) -> impl Instance<Commands = C> + Captures<Tag> {
    _Instance::new(handle, commands, tag)
}

/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyInstance.html>
impl<C, X, T> Drop for _Instance<C, X, T>
where
    C: DestroyInstance<X>,
{
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
