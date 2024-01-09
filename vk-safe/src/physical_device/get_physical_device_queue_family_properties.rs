use std::convert::TryInto;
use std::marker::PhantomData;

use super::*;

use crate::error::Error;
use crate::instance_type::Instance;
use crate::scope::ScopeId;
use crate::type_conversions::{SafeTransmute, TransmuteRef};

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceQueueFamilyProperties;

use std::fmt;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html
*/
impl<S, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn get_physical_device_queue_family_properties<A: ArrayStorage<vk::QueueFamilyProperties>>(
        &self,
        mut storage: A,
    ) -> Result<QueueFamilies<S, A>, Error>
    where
        I::Commands: GetPhysicalDeviceQueueFamilyProperties,
    {
        let families = enumerator_code2!(self.instance.commands.GetPhysicalDeviceQueueFamilyProperties().get_fptr(); (self.handle) -> storage)?;
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
        check_vuids::cur_description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "pQueueFamilyPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::cur_description! {
        "If the value referenced by pQueueFamilyPropertyCount is not 0, and pQueueFamilyProperties"
        "is not NULL, pQueueFamilyProperties must be a valid pointer to an array of pQueueFamilyPropertyCount"
        "VkQueueFamilyProperties structures"
        }

        // enumerator_code2!
    }
};

/// Properties for queue families by family index
///
/// The index of each QueueFamilyProperties is the queue family index.
pub struct QueueFamilies<S, A: ArrayStorage<vk::QueueFamilyProperties>> {
    families: A::InitStorage,
    _scope: PhantomData<S>,
}

impl<S, A: ArrayStorage<vk::QueueFamilyProperties>> QueueFamilies<S, A> {
    pub fn config_scope(&self, f: impl for<'s> FnOnce(QueueConfigScope<'s, S>)) {
        f(QueueConfigScope {
            families: self.families.as_ref(),
            _id: Default::default(),
            _pd: PhantomData,
        })
    }
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

#[repr(transparent)]
pub struct QueueFamiliesRef<S> {
    _scope: PhantomData<S>,
    families: [vk::QueueFamilyProperties],
}
unsafe impl<S> SafeTransmute<QueueFamiliesRef<S>> for [vk::QueueFamilyProperties] {}

impl<S> std::ops::Deref for QueueFamiliesRef<S> {
    type Target = [vk::QueueFamilyProperties];

    fn deref(&self) -> &Self::Target {
        &self.families
    }
}

pub struct QueueConfigScope<'scope, S> {
    families: &'scope [vk::QueueFamilyProperties],
    _id: ScopeId<'scope>,
    _pd: PhantomData<S>,
}

impl<S> std::ops::Deref for QueueConfigScope<'_, S> {
    type Target = [vk::QueueFamilyProperties];

    fn deref(&self) -> &Self::Target {
        self.families
    }
}

impl<'scope, S> IntoIterator for QueueConfigScope<'scope, S> {
    type Item = QueueFamily<'scope, (S, Self)>;

    type IntoIter = QueueFamilyIter<'scope, (S, Self)>;

    fn into_iter(self) -> Self::IntoIter {
        QueueFamilyIter {
            iter: self.families.as_ref().iter().enumerate(),
            _scope: PhantomData,
        }
    }
}

pub struct QueueFamilyIter<'a, S> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, vk::QueueFamilyProperties>>,
    _scope: PhantomData<S>,
}

impl<'a, S> Iterator for QueueFamilyIter<'a, S> {
    type Item = QueueFamily<'a, S>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, properties) = self.iter.next()?;
        let family_index: u32 = index.try_into().ok()?;
        Some(QueueFamily {
            properties,
            family_index,
            _scope: PhantomData,
        })
    }
}

pub struct QueueFamily<'a, S> {
    properties: &'a vk::QueueFamilyProperties,
    pub family_index: u32,
    _scope: PhantomData<S>,
}

impl<S> std::ops::Deref for QueueFamily<'_, S> {
    type Target = vk::QueueFamilyProperties;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}
