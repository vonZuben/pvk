use super::*;
use crate::instance::InstanceConfig;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;

use vk_safe_sys::validation::GetPhysicalDeviceQueueFamilyProperties::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C>
where
    C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceQueueFamilyProperties>,
{
    pub fn get_physical_device_queue_family_properties<S: EnumeratorStorage<QueueFamilyProperties>>(&self, mut storage: S) -> QueueFamilies<S> {
        let families = enumerator_code_non_fail!(self.handle, self.instance.feature_commands; () -> storage);
        QueueFamilies { families }
    }
}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkGetPhysicalDeviceQueueFamilyProperties_physicalDevice_parameter: () = {
        // PhysicalDevice
    };

    const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter : ( ) = {
        // enumerator_code2
    };

    const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter: () = {
        // enumerator_code2
    };
}

check_vuid_defs!(
    pub const VUID_vkGetPhysicalDeviceQueueFamilyProperties_physicalDevice_parameter:
            &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        pub const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter : & 'static [ u8 ] = "pQueueFamilyPropertyCount must be a valid pointer to a uint32_t value" . as_bytes ( ) ;
        pub const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter : & 'static [ u8 ] = "If the value referenced by pQueueFamilyPropertyCount is not 0, and pQueueFamilyProperties is not NULL, pQueueFamilyProperties must be a valid pointer to an array of pQueueFamilyPropertyCount VkQueueFamilyProperties structures" . as_bytes ( ) ;
);

simple_struct_wrapper!(QueueFamilyProperties);

impl fmt::Debug for QueueFamilyProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

/// Properties for queue families by family index
///
/// ths implements Deref<Target = [QueueFamilyProperties]>
/// the index of each QueueFamilyProperties is the queue family index
///
/// this is a wrapper type to ensure that the array of QueueFamilyProperties is not mutated
/// since the relationship between the properties and family index is an important invariant
pub struct QueueFamilies<S: EnumeratorStorage<QueueFamilyProperties>> {
    families: S::InitStorage,
}

impl<S: EnumeratorStorage<QueueFamilyProperties>> std::ops::Deref for QueueFamilies<S> {
    type Target = [QueueFamilyProperties];

    fn deref(&self) -> &Self::Target {
        self.families.as_ref()
    }
}

impl<S: EnumeratorStorage<QueueFamilyProperties>> fmt::Debug for QueueFamilies<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.families.as_ref().iter()).finish()
    }
}

impl<QS: EnumeratorStorage<QueueFamilyProperties>> QueueFamilies<QS> {
    pub fn configure_create_info<'params, IS: EnumeratorStorage<DeviceQueueCreateInfo<'params>>>(
        &self,
        mut storage: IS,
        mut filter: impl for<'properties, 'initializer, 'storage> FnMut(DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage>)
    ) -> crate::physical_device::DeviceQueueCreateInfoArray<'params, IS>
    {
        let query_len = || {
            let mut protected_count = 0;
            for properties in self.families.as_ref().iter() {
                use vk::queue_flag_bits::*;
                if properties.queue_flags.contains(vk::VkBitmaskType::from_bit_type_list(bitmask!(PROTECTED_BIT))) && properties.queue_count > 1 {
                    protected_count += 1;
                }
            }
            Ok(self.families.as_ref().len() + protected_count)
        };
        storage.query_len(query_len).expect("error in configure_create_info: could not allocate storage");

        let mut initializer = crate::enumerator_storage::UninitArrayInitializer::new(storage.uninit_slice().iter_mut());
        for (index, properties) in self.families.as_ref().iter().enumerate() {
            filter(DeviceQueueCreateInfoConfiguration::new(index as u32, &mut initializer, properties));
        }

        let init_count = initializer.initialized_count();
        assert!(init_count > 0, "must configure at least one queue family");
        DeviceQueueCreateInfoArray::new(storage.finalize(init_count))
    }
}