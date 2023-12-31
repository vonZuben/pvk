use super::create_device::{
    DeviceQueueCreateInfo, DeviceQueueCreateInfoArray, DeviceQueueCreateInfoConfiguration,
};
use super::*;
use crate::error::Error;
use crate::instance_type::Instance;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceQueueFamilyProperties;

use std::fmt;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html
*/
impl<S, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn get_physical_device_queue_family_properties<A: ArrayStorage<QueueFamilyProperties<S>>>(
        &self,
        mut storage: A,
    ) -> Result<QueueFamilies<S, A>, Error>
    where
        I::Commands: GetPhysicalDeviceQueueFamilyProperties,
    {
        let families = enumerator_code2!(self.instance.commands.GetPhysicalDeviceQueueFamilyProperties().get_fptr(); (self.handle) -> storage)?;
        Ok(QueueFamilies { families })
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

simple_struct_wrapper_scoped!(QueueFamilyProperties impl Deref, Debug);

/// Properties for queue families by family index
///
/// ths implements Deref<Target = [QueueFamilyProperties]>
/// the index of each QueueFamilyProperties is the queue family index
///
/// this is a wrapper type to ensure that the array of QueueFamilyProperties is not mutated
/// since the relationship between the properties and family index is an important invariant
pub struct QueueFamilies<S, A: ArrayStorage<QueueFamilyProperties<S>>> {
    families: A::InitStorage,
}

impl<S, A: ArrayStorage<QueueFamilyProperties<S>>> std::ops::Deref for QueueFamilies<S, A> {
    type Target = [QueueFamilyProperties<S>];

    fn deref(&self) -> &Self::Target {
        self.families.as_ref()
    }
}

impl<S, A: ArrayStorage<QueueFamilyProperties<S>>> fmt::Debug for QueueFamilies<S, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.families.as_ref().iter())
            .finish()
    }
}

impl<S, QA: ArrayStorage<QueueFamilyProperties<S>>> QueueFamilies<S, QA> {
    pub fn configure_create_info<'params, IA: ArrayStorage<DeviceQueueCreateInfo<'params, S>>>(
        &self,
        mut storage: IA,
        mut filter: impl for<'properties, 'initializer, 'storage> FnMut(
            DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage, S>,
        ),
    ) -> DeviceQueueCreateInfoArray<'params, IA, S> {
        let len = || {
            let mut protected_count = 0;
            for properties in self.families.as_ref().iter() {
                use vk::queue_flag_bits::*;
                if properties.queue_flags.contains(PROTECTED_BIT) && properties.queue_count > 1 {
                    protected_count += 1;
                }
            }
            Ok(self.families.as_ref().len() + protected_count)
        };
        storage
            .allocate(len)
            .expect("error in configure_create_info: could not allocate storage");

        let mut initializer =
            crate::array_storage::UninitArrayInitializer::new(storage.uninit_slice().iter_mut());
        for (index, properties) in self.families.as_ref().iter().enumerate() {
            filter(DeviceQueueCreateInfoConfiguration::new(
                index as u32,
                &mut initializer,
                properties,
            ));
        }

        let init_count = initializer.initialized_count();
        assert!(init_count > 0, "must configure at least one queue family");
        DeviceQueueCreateInfoArray::new(storage.finalize(init_count))
    }
}
