/*!
Query the queue family properties of the PhysicalDevice

use the [`get_physical_device_queue_family_properties`](ScopedPhysicalDevice::get_physical_device_queue_family_properties) method on a scoped PhysicalDevice

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceQueueFamilyProperties.html>
*/

use std::convert::TryInto;
use std::marker::PhantomData;

use std::fmt;

use super::concrete_type::ScopedPhysicalDevice;

use crate::array_storage::ArrayStorage;
use crate::dispatchable_handles::instance::Instance;
use crate::error::Error;
use crate::scope::Tag;
use crate::type_conversions::{SafeTransmute, TransmuteRef};

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceQueueFamilyProperties;

impl<S, I: Instance> ScopedPhysicalDevice<S, I>
where
    I::Context: GetPhysicalDeviceQueueFamilyProperties,
{
    /**
    Query the queue family properties of the PhysicalDevice

    Must provide the storage space to return the properties to.

    # Scope
    The returned [`QueueFamilies`] **must** be scoped with
    [`scope!`](crate::vk::scope) in order to be usable for
    device Queue configuration. See ...

    ```rust
    # use vk_safe::vk;
    # vk::device_context!(D: VERSION_1_0);
    # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    #   (physical_device: P) {
    let queue_family_properties = physical_device.get_physical_device_queue_family_properties(Vec::new());
    # }
    ```
    */
    pub fn get_physical_device_queue_family_properties<
        A: ArrayStorage<vk::QueueFamilyProperties>,
    >(
        &self,
        mut storage: A,
    ) -> Result<QueueFamilies<S, A>, Error> {
        let families = enumerator_code2!(self.instance.context.GetPhysicalDeviceQueueFamilyProperties().get_fptr(); (self.handle) -> storage)?;
        Ok(QueueFamilies {
            families,
            _scope: PhantomData,
        })
    }
}

// ensured by PhysicalDevice and enumerator_code2!()
const _VUID: () = {
    check_vuids::check_vuids!(GetPhysicalDeviceQueueFamilyProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pQueueFamilyPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the value referenced by pQueueFamilyPropertyCount is not 0, and pQueueFamilyProperties"
        "is not NULL, pQueueFamilyProperties must be a valid pointer to an array of pQueueFamilyPropertyCount"
        "VkQueueFamilyProperties structures"
        }

        // enumerator_code2!
    }
};

/// Properties for queue families by family index
///
/// This is a wrapper for an array of [`QueueFamilyProperties`](vk_safe_sys::QueueFamilyProperties).
/// The index of the properties is the queue family index. The wrapper ensures
/// that the index relationship is maintained.
///
/// # Device configuration
/// A key step in configuring a logical Device is configuring the Queues
/// with [`DeviceQueueCreateInfo`](crate::vk::DeviceQueueCreateInfo).
/// The `QueueFamilyProperties` are needed to determine what the Queues
/// of each family can do, and how many Queues can be made for the family.
pub struct QueueFamilies<S, A: ArrayStorage<vk::QueueFamilyProperties>> {
    families: A::InitStorage,
    _scope: PhantomData<S>,
}

impl<S, A: ArrayStorage<vk::QueueFamilyProperties>> std::ops::Deref for QueueFamilies<S, A> {
    type Target = QueueFamiliesRef<S>;

    fn deref(&self) -> &Self::Target {
        self.families.as_ref().transmute_ref()
    }
}

impl<S, A: ArrayStorage<vk::QueueFamilyProperties>> fmt::Debug for QueueFamilies<S, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.families.as_ref().iter())
            .finish()
    }
}

/// Reference to QueueFamilies
///
/// This is mainly for abstracting away the [`ArrayStorage`] generics
/// of [`QueueFamilies`].
#[repr(transparent)]
pub struct QueueFamiliesRef<S> {
    _scope: PhantomData<S>,
    families: [vk::QueueFamilyProperties],
}
unsafe impl<S> SafeTransmute<QueueFamiliesRef<S>> for [vk::QueueFamilyProperties] {}

impl<S> QueueFamiliesRef<S> {
    /// Iterate over [`QueueFamilyProperties`] with a provided [`Tag`]
    ///
    /// The `tag` ensures that all `properties` are related to the same
    /// collection. The `properties` are not Copy/Clone to ensure that each one
    /// can only be used once per tag. This is because you may only configure
    /// each QueueFamily once per logical Device created.
    pub fn properties_iter<'id>(&self, tag: Tag<'id>) -> QueueFamilyIter<(S, Tag<'id>)> {
        let _ = tag;
        QueueFamilyIter {
            iter: self.iter().enumerate(),
            _scope: PhantomData,
        }
    }
}

impl<S> std::ops::Deref for QueueFamiliesRef<S> {
    type Target = [vk::QueueFamilyProperties];

    fn deref(&self) -> &Self::Target {
        &self.families
    }
}

/// An iterator over QueueFamilyProperties
pub struct QueueFamilyIter<'a, S> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, vk::QueueFamilyProperties>>,
    _scope: PhantomData<S>,
}

impl<'a, S> Iterator for QueueFamilyIter<'a, S> {
    type Item = QueueFamilyProperties<'a, S>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, properties) = self.iter.next()?;
        let family_index: u32 = index.try_into().ok()?;
        Some(QueueFamilyProperties {
            properties,
            family_index,
            _scope: PhantomData,
        })
    }
}

/// Properties of a Queue Family
///
/// Acquire by iterating over [`QueueFamilies`] via
/// [`properties_iter`](QueueFamiliesRef::properties_iter).
///
/// Each Queue family may only be configured 0 or 1 times.
/// A `tag` is needed to ensure that all `QueueFamilyProperties`
/// are related to the same collection. They are also
/// not Copy/Clone to ensure each one can only be used once.
pub struct QueueFamilyProperties<'a, S> {
    properties: &'a vk::QueueFamilyProperties,
    pub family_index: u32,
    _scope: PhantomData<S>,
}

impl<S> std::ops::Deref for QueueFamilyProperties<'_, S> {
    type Target = vk::QueueFamilyProperties;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}
