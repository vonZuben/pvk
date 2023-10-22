use super::*;
use crate::device::{Config, DeviceType};
use crate::error::Error;
use crate::instance::Instance;
use vk_safe_sys as vk;

use crate::safe_interface::type_conversions::transmute_array;

use std::fmt;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use vk::commands::{LoadCommands, Version};
use vk::has_command::{CreateDevice, DestroyDevice};

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateDevice.html
*/
impl<'scope, I: Instance> ScopedPhysicalDeviceType<'scope, I> {
    pub fn create_device<Create, Destroy, Commands>(
        &self,
        create_info: &DeviceCreateInfo<'_, Commands>,
    ) -> Result<DeviceType<Config<Destroy, Commands>, Self>, Error>
    where
        I::Commands: CreateDevice<Create>,
        Commands: DestroyDevice<Destroy> + LoadCommands + Version,
    {
        let mut device = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.commands.CreateDevice().get_fptr()(
                self.handle,
                &create_info.inner,
                std::ptr::null(),
                device.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(DeviceType::load_commands(device.assume_init())?)
        }
    }
}

//===========InstanceCreateInfo
pub struct DeviceCreateInfo<'a, C> {
    pub(crate) inner: vk::DeviceCreateInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
}

impl<'a> DeviceCreateInfo<'a, ()> {
    pub const fn new<Commands>(
        queue_create_info: &'a [DeviceQueueCreateInfo],
    ) -> DeviceCreateInfo<'a, Commands> {
        check_vuid_defs2!( DeviceCreateInfo
            pub const VUID_VkDeviceCreateInfo_queueFamilyIndex_00372: &'static [u8] = "".as_bytes();
            // VUID_VkDeviceCreateInfo_queueFamilyIndex_00372 appears to be a mistake since no definition is provided
            pub const VUID_VkDeviceCreateInfo_sType_sType: &'static [u8] =
                "sType must be VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO".as_bytes();
            // s_type is set below
            pub const VUID_VkDeviceCreateInfo_pNext_pNext : & 'static [ u8 ] = "Each pNext member of any structure (including this one) in the pNext chain must be either NULL or a pointer to a valid instance of VkDeviceGroupDeviceCreateInfo, VkDeviceMemoryOverallocationCreateInfoAMD, VkPhysicalDevice16BitStorageFeatures, VkPhysicalDevice8BitStorageFeaturesKHR, VkPhysicalDeviceASTCDecodeFeaturesEXT, VkPhysicalDeviceBlendOperationAdvancedFeaturesEXT, VkPhysicalDeviceBufferAddressFeaturesEXT, VkPhysicalDeviceComputeShaderDerivativesFeaturesNV, VkPhysicalDeviceConditionalRenderingFeaturesEXT, VkPhysicalDeviceCornerSampledImageFeaturesNV, VkPhysicalDeviceDescriptorIndexingFeaturesEXT, VkPhysicalDeviceExclusiveScissorFeaturesNV, VkPhysicalDeviceFeatures2, VkPhysicalDeviceFloat16Int8FeaturesKHR, VkPhysicalDeviceFragmentDensityMapFeaturesEXT, VkPhysicalDeviceFragmentShaderBarycentricFeaturesNV, VkPhysicalDeviceInlineUniformBlockFeaturesEXT, VkPhysicalDeviceMemoryPriorityFeaturesEXT, VkPhysicalDeviceMeshShaderFeaturesNV, VkPhysicalDeviceMultiviewFeatures, VkPhysicalDeviceProtectedMemoryFeatures, VkPhysicalDeviceRepresentativeFragmentTestFeaturesNV, VkPhysicalDeviceSamplerYcbcrConversionFeatures, VkPhysicalDeviceScalarBlockLayoutFeaturesEXT, VkPhysicalDeviceShaderAtomicInt64FeaturesKHR, VkPhysicalDeviceShaderDrawParameterFeatures, VkPhysicalDeviceShaderImageFootprintFeaturesNV, VkPhysicalDeviceShadingRateImageFeaturesNV, VkPhysicalDeviceTransformFeedbackFeaturesEXT, VkPhysicalDeviceVariablePointerFeatures, VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT, or VkPhysicalDeviceVulkanMemoryModelFeaturesKHR" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when add p_next support
            }
            pub const VUID_VkDeviceCreateInfo_sType_unique: &'static [u8] =
                "Each sType member in the pNext chain must be unique".as_bytes();
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when add p_next support
            }
            pub const VUID_VkDeviceCreateInfo_flags_zerobitmask: &'static [u8] =
                "flags must be 0".as_bytes();
            // flags is set below
            pub const VUID_VkDeviceCreateInfo_pQueueCreateInfos_parameter : & 'static [ u8 ] = "pQueueCreateInfos must be a valid pointer to an array of queueCreateInfoCount valid VkDeviceQueueCreateInfo structures" . as_bytes ( ) ;
            CHECK {
                // the queue create infos are created in a DeviceQueueCreateInfoArray via DeviceQueueCreateInfoConfiguration which ensure valid infos
                // also rust references are used and are valid, and the array len is used for count
            }
            pub const VUID_VkDeviceCreateInfo_ppEnabledLayerNames_parameter : & 'static [ u8 ] = "If enabledLayerCount is not 0, ppEnabledLayerNames must be a valid pointer to an array of enabledLayerCount null-terminated UTF-8 strings" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when support layers
            }
            pub const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_parameter : & 'static [ u8 ] = "If enabledExtensionCount is not 0, ppEnabledExtensionNames must be a valid pointer to an array of enabledExtensionCount null-terminated UTF-8 strings" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when support extensions
            }
            pub const VUID_VkDeviceCreateInfo_pEnabledFeatures_parameter : & 'static [ u8 ] = "If pEnabledFeatures is not NULL, pEnabledFeatures must be a valid pointer to a valid VkPhysicalDeviceFeatures structure" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when support features
            }
            pub const VUID_VkDeviceCreateInfo_queueCreateInfoCount_arraylength: &'static [u8] =
                "queueCreateInfoCount must be greater than 0".as_bytes();
            CHECK {
                assert!(queue_create_info.len() > 0);
            }
            pub const VUID_VkDeviceCreateInfo_pNext_00373 : & 'static [ u8 ] = "If the pNext chain includes a VkPhysicalDeviceFeatures2 structure, then pEnabledFeatures must be NULL" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when add p_next support
            }
            pub const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_01840: &'static [u8] =
                "ppEnabledExtensionNames must not contain VK_AMD_negative_viewport_height".as_bytes();
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when support extensions
            }
            pub const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_00374 : & 'static [ u8 ] = "ppEnabledExtensionNames must not contain both VK_KHR_maintenance1 and VK_AMD_negative_viewport_height" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when support extensions
            }
        );

        DeviceCreateInfo {
            inner: vk::DeviceCreateInfo {
                s_type: vk::StructureType::DEVICE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceCreateFlags::empty(),
                queue_create_info_count: queue_create_info.len() as u32,
                p_queue_create_infos: transmute_array(queue_create_info).as_ptr(),
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: 0,
                pp_enabled_extension_names: std::ptr::null(),
                p_enabled_features: std::ptr::null(),
            },
            _config: PhantomData,
            _refs: PhantomData,
        }
    }
}

/// A safe to use [vk::DeviceQueueCreateInfo]
#[repr(transparent)]
pub struct DeviceQueueCreateInfo<'a> {
    inner: vk::DeviceQueueCreateInfo,
    _refs: PhantomData<&'a ()>,
}

unsafe impl crate::safe_interface::type_conversions::SafeTransmute<vk::DeviceQueueCreateInfo>
    for DeviceQueueCreateInfo<'_>
{
}

impl DeviceQueueCreateInfo<'_> {
    array!(queue_priorities, p_queue_priorities, queue_count, f32);
}

impl std::ops::Deref for DeviceQueueCreateInfo<'_> {
    type Target = vk::DeviceQueueCreateInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl fmt::Debug for DeviceQueueCreateInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceQueueCreateInfo")
            .field("flags", &self.inner.flags)
            .field("p_next", &"TODO")
            .field("queue_family_index", &self.inner.queue_family_index)
            .field("queue_count", &self.inner.queue_count)
            .field("queue_priorities", &self.queue_priorities())
            .finish()
    }
}

pub struct DeviceQueueCreateInfoArray<'a, S: ArrayStorage<DeviceQueueCreateInfo<'a>>> {
    infos: S::InitStorage,
    _a: PhantomData<&'a ()>,
}

impl<'a, S: ArrayStorage<DeviceQueueCreateInfo<'a>>> fmt::Debug
    for DeviceQueueCreateInfoArray<'a, S>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.infos.as_ref().iter()).finish()
    }
}

impl<'a, S: ArrayStorage<DeviceQueueCreateInfo<'a>>> std::ops::Deref
    for DeviceQueueCreateInfoArray<'a, S>
{
    type Target = [DeviceQueueCreateInfo<'a>];

    fn deref(&self) -> &Self::Target {
        &self.infos.as_ref()
    }
}

impl<'a, S: ArrayStorage<DeviceQueueCreateInfo<'a>>> DeviceQueueCreateInfoArray<'a, S> {
    pub(crate) fn new(infos: S::InitStorage) -> Self {
        Self {
            infos,
            _a: PhantomData,
        }
    }
}

/// an array of queue priorities
/// len must be > 0
/// all values must fall in 0.0..=1.0
#[derive(Clone, Copy)]
pub struct QueuePriorities<'a> {
    priorities: &'a [f32],
}

impl<'a> QueuePriorities<'a> {
    pub fn new(priorities: &'a [f32]) -> Self {
        assert!(priorities.as_ref().len() > 0);
        for p in priorities.as_ref().iter().copied() {
            assert!(0.0 <= p && p <= 1.0)
        }
        unsafe { Self::new_unchecked(priorities) }
    }
    /// Safety
    /// must ensure that priorities.as_ref().len() > 0, and all values are in the range 0.0..=1.0
    pub unsafe fn new_unchecked(priorities: &'a [f32]) -> Self {
        Self { priorities }
    }
    pub fn with_num_queues(&self, num_queues: usize) -> Self {
        assert!(num_queues > 0);
        unsafe { QueuePriorities::new_unchecked(&self.priorities.as_ref()[..num_queues]) }
    }
    pub fn len(&self) -> usize {
        self.priorities.as_ref().len()
    }
    pub fn as_ptr(&self) -> *const f32 {
        self.priorities.as_ref().as_ptr()
    }
}

/// Builder for [DeviceQueueCreateInfo]
/// initially created with sane defaults
/// user must provide their own p_queue_priorities
pub struct DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage, 'scope>
{
    family_index: u32,
    to_write: &'initializer mut crate::array_storage::UninitArrayInitializer<
        'storage,
        DeviceQueueCreateInfo<'params>,
    >,
    pub family_properties: &'properties QueueFamilyProperties<'scope>,
}

impl<'params, 'properties, 'initializer, 'storage, 'scope>
    DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage, 'scope>
{
    /// internal only method to be called from [QueueFamilies::create_info_builder_iter]
    /// should pass the current queue_family_index, and set queue_count to max possible
    pub(crate) fn new(
        family_index: u32,
        to_write: &'initializer mut crate::array_storage::UninitArrayInitializer<
            'storage,
            DeviceQueueCreateInfo<'params>,
        >,
        family_properties: &'properties QueueFamilyProperties<'scope>,
    ) -> Self {
        Self {
            family_index,
            to_write,
            family_properties,
        }
    }
    /// configure DeviceQueueCreateInfo for this family to use priorities.len queues with the given priorities, and with given flag
    pub fn push_config(
        self,
        priorities: QueuePriorities,
        flags: vk::DeviceQueueCreateFlags,
    ) -> crate::array_storage::InitResult {
        check_vuid_defs2!( DeviceQueueCreateInfo
            pub const VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381 : & 'static [ u8 ] = "queueFamilyIndex must be less than pQueueFamilyPropertyCount returned by vkGetPhysicalDeviceQueueFamilyProperties" . as_bytes ( ) ;
            CHECK {
                // Self.family_index should be valid
            }
            pub const VUID_VkDeviceQueueCreateInfo_queueCount_00382 : & 'static [ u8 ] = "queueCount must be less than or equal to the queueCount member of the VkQueueFamilyProperties structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties in the pQueueFamilyProperties[queueFamilyIndex]" . as_bytes ( ) ;
            CHECK {
                assert!(priorities.len() <= self.family_properties.queue_count as usize);
            }
            pub const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: &'static [u8] =
                "Each element of pQueuePriorities must be between 0.0 and 1.0 inclusive".as_bytes();
            CHECK {
                // this is verified by QueuePriorities::new()
            }
            pub const VUID_VkDeviceQueueCreateInfo_sType_sType: &'static [u8] =
                "sType must be VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO".as_bytes();
            // s_type set below
            pub const VUID_VkDeviceQueueCreateInfo_pNext_pNext : & 'static [ u8 ] = "pNext must be NULL or a pointer to a valid instance of VkDeviceQueueGlobalPriorityCreateInfoEXT" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when add p_next support
            }
            pub const VUID_VkDeviceQueueCreateInfo_flags_parameter: &'static [u8] =
                "flags must be a valid combination of VkDeviceQueueCreateFlagBits values".as_bytes();
            CHECK {
                assert!(!flags.contains(vk::DeviceQueueCreateFlags::PROTECTED_BIT), "must not push_config with PROTECTED_BIT. use push_config_with_protected instead");
            }
            pub const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: &'static [u8] =
                "pQueuePriorities must be a valid pointer to an array of queueCount float values"
                    .as_bytes();
            CHECK {
                // guaranteed by QueuePriorities
            }
            pub const VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: &'static [u8] =
                "queueCount must be greater than 0".as_bytes();
            CHECK {
                // guaranteed by QueuePriorities
            }
        );

        let info = DeviceQueueCreateInfo {
            inner: vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                flags: flags,
                p_next: std::ptr::null(),
                queue_family_index: self.family_index,
                queue_count: priorities.len() as u32, // the assert already confirms no overflow from conversion
                p_queue_priorities: priorities.as_ptr(),
            },
            _refs: PhantomData,
        };
        self.to_write.push(info)
    }
    /// like [configure], but will configure two [DeviceQueueCreateInfo] where one must be protected, and the other not
    /// based on rules in:
    /// VUID-VkDeviceCreateInfo-queueFamilyIndex-02802
    /// VUID-VkDeviceCreateInfo-pQueueCreateInfos-06755
    pub fn push_config_with_protected<A: AsRef<[f32]> + 'params>(
        self,
        priorities_for_non_protected: QueuePriorities,
        flags_for_non_protected: vk::DeviceQueueCreateFlags,
        priorities_for_protected: QueuePriorities,
        flags_for_protected: vk::DeviceQueueCreateFlags,
    ) -> crate::array_storage::InitResult {
        check_vuid_defs2!( DeviceQueueCreateInfo
            pub const VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381 : & 'static [ u8 ] = "queueFamilyIndex must be less than pQueueFamilyPropertyCount returned by vkGetPhysicalDeviceQueueFamilyProperties" . as_bytes ( ) ;
            CHECK {
                // Self.family_index should be valid
            }
            pub const VUID_VkDeviceQueueCreateInfo_queueCount_00382 : & 'static [ u8 ] = "queueCount must be less than or equal to the queueCount member of the VkQueueFamilyProperties structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties in the pQueueFamilyProperties[queueFamilyIndex]" . as_bytes ( ) ;
            CHECK {
                assert!(priorities_for_non_protected.len() + priorities_for_protected.len() <= self.family_properties.queue_count as usize);
            }
            pub const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: &'static [u8] =
                "Each element of pQueuePriorities must be between 0.0 and 1.0 inclusive".as_bytes();
            CHECK {
                // guaranteed by QueuePriorities
            }
            pub const VUID_VkDeviceQueueCreateInfo_sType_sType: &'static [u8] =
                "sType must be VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO".as_bytes();
            // s_type set below
            pub const VUID_VkDeviceQueueCreateInfo_pNext_pNext : & 'static [ u8 ] = "pNext must be NULL or a pointer to a valid instance of VkDeviceQueueGlobalPriorityCreateInfoEXT" . as_bytes ( ) ;
            CHECK {
                // ******************************************
                // *****************TODO*********************
                // ******************************************
                // need check when add p_next support
            }
            pub const VUID_VkDeviceQueueCreateInfo_flags_parameter: &'static [u8] =
                "flags must be a valid combination of VkDeviceQueueCreateFlagBits values".as_bytes();
            CHECK {
                assert!(self.family_properties.queue_flags.contains(vk::QueueFlags::PROTECTED_BIT));
                assert!(!flags_for_non_protected.contains(vk::DeviceQueueCreateFlags::PROTECTED_BIT), "flags_for_non_protected should not include PROTECTED_BIT");
                assert!(flags_for_protected.contains(vk::DeviceQueueCreateFlags::PROTECTED_BIT), "flags_for_protected must include PROTECTED_BIT");
            }
            pub const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: &'static [u8] =
                "pQueuePriorities must be a valid pointer to an array of queueCount float values"
                    .as_bytes();
            CHECK {
                // guaranteed by QueuePriorities
            }
            pub const VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: &'static [u8] =
                "queueCount must be greater than 0".as_bytes();
            CHECK {
                // guaranteed by QueuePriorities
            }
        );

        if priorities_for_non_protected.len() > 0 {
            let non_protected_info = DeviceQueueCreateInfo {
                inner: vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    flags: flags_for_non_protected,
                    p_next: std::ptr::null(),
                    queue_family_index: self.family_index,
                    queue_count: priorities_for_non_protected.len() as u32, // the assert already confirms no overflow from conversion
                    p_queue_priorities: priorities_for_non_protected.as_ptr(),
                },
                _refs: PhantomData,
            };
            self.to_write.push(non_protected_info)?;
        }

        if priorities_for_protected.len() > 0 {
            let protected_info = DeviceQueueCreateInfo {
                inner: vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    flags: flags_for_protected,
                    p_next: std::ptr::null(),
                    queue_family_index: self.family_index,
                    queue_count: priorities_for_protected.len() as u32, // the assert already confirms no overflow from conversion
                    p_queue_priorities: priorities_for_protected.as_ptr(),
                },
                _refs: PhantomData,
            };
            self.to_write.push(protected_info)?;
        }

        Ok(())
    }
}
