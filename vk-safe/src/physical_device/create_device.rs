use super::*;
use vk_safe_sys as vk;
use krs_hlist::Get;
use crate::instance::{InstanceConfig, Instance};
use crate::device::{Device, DeviceConfig};

use vk::VkEnumVariant;
use vk::BitList;

use crate::safe_interface::type_conversions::TransmuteArray;

use std::mem::MaybeUninit;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct TempError;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateDevice.html
*/
impl<'instance, IC: InstanceConfig> PhysicalDevice<'instance, IC> where IC::InstanceCommands: vk::GetCommand<vk::CreateDevice> {
    pub fn create_device<DC: DeviceConfig>(&self, create_info: &DeviceCreateInfo<'_, DC>) -> Result<Device<'instance, Instance<IC>, DC>, TempError> {
        let mut device = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.feature_commands.get().get_fptr()(
                self.handle,
                &create_info.inner,
                std::ptr::null(),
                device.as_mut_ptr()
            );
            if res.is_err() {
                Err(TempError)
            }
            else {
                Ok(
                    Device::load_commands(device.assume_init()).map_err(|_|TempError)?
                )
            }
        }
    }
}

//===========InstanceCreateInfo
pub struct DeviceCreateInfo <'a, C: DeviceConfig> {
    pub(crate) inner: vk::DeviceCreateInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
}

impl<'a, C: DeviceConfig> DeviceCreateInfo<'a, C> {
    pub fn new(_config: C, queue_create_info: &'a [DeviceQueueCreateInfo]) -> Self {
        assert!(queue_create_info.len() > 0);
        validate_device_create_info::Validation::validate();
        Self {
            inner: vk::DeviceCreateInfo {
                s_type: vk::structure_type::DEVICE_CREATE_INFO.as_enum(),
                p_next: std::ptr::null(),
                flags: unsafe { vk::DeviceCreateFlags::empty() }, // VUID_VkDeviceCreateInfo_flags_zerobitmask
                queue_create_info_count: queue_create_info.len() as u32,
                p_queue_create_infos: queue_create_info.safe_transmute().as_ptr(),
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

mod validate_device_create_info {
    use vk_safe_sys::validation::DeviceCreateInfo::*;

    pub struct Validation;

    impl Validation {
        pub(crate) fn validate() {
            validate(Self)
        }
    }

    #[allow(non_upper_case_globals)]
    impl Vuids for Validation {
        const VUID_VkDeviceCreateInfo_queueFamilyIndex_00372: () = {
            // This VUID has no description in the current vuid json file
        };

        const VUID_VkDeviceCreateInfo_sType_sType: () = {
            // handled in new()
        };

        const VUID_VkDeviceCreateInfo_pNext_pNext: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when add p_next support
        };

        const VUID_VkDeviceCreateInfo_sType_unique: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when add p_next support
        };

        const VUID_VkDeviceCreateInfo_flags_zerobitmask: () = {
            // set in new()
        };

        const VUID_VkDeviceCreateInfo_pQueueCreateInfos_parameter: () = {
            // the queue create infos are created in a DeviceQueueCreateInfoArray via DeviceQueueCreateInfoConfiguration which ensure valid infos
            // also rust references are used and are valid, and the array len is used for count
        };

        const VUID_VkDeviceCreateInfo_ppEnabledLayerNames_parameter: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when support layers
        };

        const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_parameter: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when support extensions
        };

        const VUID_VkDeviceCreateInfo_pEnabledFeatures_parameter: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when support features
        };

        const VUID_VkDeviceCreateInfo_queueCreateInfoCount_arraylength: () = {
            // checked via assert!()
        };

        const VUID_VkDeviceCreateInfo_pNext_00373: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when add p_next support
        };

        const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_01840: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when support extensions
        };

        const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_00374: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when support extensions
        };
    }

    check_vuid_defs!(
        pub const VUID_VkDeviceCreateInfo_queueFamilyIndex_00372: &'static [u8] = "".as_bytes();
        pub const VUID_VkDeviceCreateInfo_sType_sType: &'static [u8] =
            "sType must be VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO".as_bytes();
        pub const VUID_VkDeviceCreateInfo_pNext_pNext : & 'static [ u8 ] = "Each pNext member of any structure (including this one) in the pNext chain must be either NULL or a pointer to a valid instance of VkDeviceGroupDeviceCreateInfo, VkDeviceMemoryOverallocationCreateInfoAMD, VkPhysicalDevice16BitStorageFeatures, VkPhysicalDevice8BitStorageFeaturesKHR, VkPhysicalDeviceASTCDecodeFeaturesEXT, VkPhysicalDeviceBlendOperationAdvancedFeaturesEXT, VkPhysicalDeviceBufferAddressFeaturesEXT, VkPhysicalDeviceComputeShaderDerivativesFeaturesNV, VkPhysicalDeviceConditionalRenderingFeaturesEXT, VkPhysicalDeviceCornerSampledImageFeaturesNV, VkPhysicalDeviceDescriptorIndexingFeaturesEXT, VkPhysicalDeviceExclusiveScissorFeaturesNV, VkPhysicalDeviceFeatures2, VkPhysicalDeviceFloat16Int8FeaturesKHR, VkPhysicalDeviceFragmentDensityMapFeaturesEXT, VkPhysicalDeviceFragmentShaderBarycentricFeaturesNV, VkPhysicalDeviceInlineUniformBlockFeaturesEXT, VkPhysicalDeviceMemoryPriorityFeaturesEXT, VkPhysicalDeviceMeshShaderFeaturesNV, VkPhysicalDeviceMultiviewFeatures, VkPhysicalDeviceProtectedMemoryFeatures, VkPhysicalDeviceRepresentativeFragmentTestFeaturesNV, VkPhysicalDeviceSamplerYcbcrConversionFeatures, VkPhysicalDeviceScalarBlockLayoutFeaturesEXT, VkPhysicalDeviceShaderAtomicInt64FeaturesKHR, VkPhysicalDeviceShaderDrawParameterFeatures, VkPhysicalDeviceShaderImageFootprintFeaturesNV, VkPhysicalDeviceShadingRateImageFeaturesNV, VkPhysicalDeviceTransformFeedbackFeaturesEXT, VkPhysicalDeviceVariablePointerFeatures, VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT, or VkPhysicalDeviceVulkanMemoryModelFeaturesKHR" . as_bytes ( ) ;
        pub const VUID_VkDeviceCreateInfo_sType_unique: &'static [u8] =
            "Each sType member in the pNext chain must be unique".as_bytes();
        pub const VUID_VkDeviceCreateInfo_flags_zerobitmask: &'static [u8] =
            "flags must be 0".as_bytes();
        pub const VUID_VkDeviceCreateInfo_pQueueCreateInfos_parameter : & 'static [ u8 ] = "pQueueCreateInfos must be a valid pointer to an array of queueCreateInfoCount valid VkDeviceQueueCreateInfo structures" . as_bytes ( ) ;
        pub const VUID_VkDeviceCreateInfo_ppEnabledLayerNames_parameter : & 'static [ u8 ] = "If enabledLayerCount is not 0, ppEnabledLayerNames must be a valid pointer to an array of enabledLayerCount null-terminated UTF-8 strings" . as_bytes ( ) ;
        pub const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_parameter : & 'static [ u8 ] = "If enabledExtensionCount is not 0, ppEnabledExtensionNames must be a valid pointer to an array of enabledExtensionCount null-terminated UTF-8 strings" . as_bytes ( ) ;
        pub const VUID_VkDeviceCreateInfo_pEnabledFeatures_parameter : & 'static [ u8 ] = "If pEnabledFeatures is not NULL, pEnabledFeatures must be a valid pointer to a valid VkPhysicalDeviceFeatures structure" . as_bytes ( ) ;
        pub const VUID_VkDeviceCreateInfo_queueCreateInfoCount_arraylength: &'static [u8] =
            "queueCreateInfoCount must be greater than 0".as_bytes();
        pub const VUID_VkDeviceCreateInfo_pNext_00373 : & 'static [ u8 ] = "If the pNext chain includes a VkPhysicalDeviceFeatures2 structure, then pEnabledFeatures must be NULL" . as_bytes ( ) ;
        pub const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_01840: &'static [u8] =
            "ppEnabledExtensionNames must not contain VK_AMD_negative_viewport_height".as_bytes();
            pub const VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_00374 : & 'static [ u8 ] = "ppEnabledExtensionNames must not contain both VK_KHR_maintenance1 and VK_AMD_negative_viewport_height" . as_bytes ( ) ;
    );
}

/// A safe to use [vk::DeviceQueueCreateInfo]
#[repr(transparent)]
pub struct DeviceQueueCreateInfo<'a> {
    inner: vk::DeviceQueueCreateInfo,
    _refs: PhantomData<&'a ()>,
}

unsafe impl crate::safe_interface::type_conversions::SafeTransmute<vk::DeviceQueueCreateInfo> for DeviceQueueCreateInfo<'_> {}

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

mod validate_device_queue_create_info {
    use vk_safe_sys::validation::DeviceQueueCreateInfo::*;

    pub struct Validation;

    #[allow(non_upper_case_globals)]
    impl Vuids for Validation {
        const VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381: () = {
            // QueueFamilies::configure_create_info api ensures valid queue family index
        };

        const VUID_VkDeviceQueueCreateInfo_queueCount_00382: () = {
            // assert!() in DeviceQueueCreateInfoConfiguration::configure
        };

        const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: () = {
            // ensured by QueuePriorities
        };

        const VUID_VkDeviceQueueCreateInfo_sType_sType: () = {
            // new()
        };

        const VUID_VkDeviceQueueCreateInfo_pNext_pNext: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when add p_next support
        };

        const VUID_VkDeviceQueueCreateInfo_flags_parameter: () = {
            // ******************************************
            // *****************TODO*********************
            // ******************************************
            // need check when add flags support
        };

        const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: () = {
            // ensured by ensured by QueuePriorities and DeviceQueueCreateInfoConfiguration::push_config/push_config_with_protected
        };

        const VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: () = {
            // ensured in QueueFamilies::configure_create_info
        };
    }

    check_vuid_defs!(
        pub const VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381 : & 'static [ u8 ] = "queueFamilyIndex must be less than pQueueFamilyPropertyCount returned by vkGetPhysicalDeviceQueueFamilyProperties" . as_bytes ( ) ;
        pub const VUID_VkDeviceQueueCreateInfo_queueCount_00382 : & 'static [ u8 ] = "queueCount must be less than or equal to the queueCount member of the VkQueueFamilyProperties structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties in the pQueueFamilyProperties[queueFamilyIndex]" . as_bytes ( ) ;
        pub const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: &'static [u8] =
            "Each element of pQueuePriorities must be between 0.0 and 1.0 inclusive".as_bytes();
        pub const VUID_VkDeviceQueueCreateInfo_sType_sType: &'static [u8] =
            "sType must be VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO".as_bytes();
        pub const VUID_VkDeviceQueueCreateInfo_pNext_pNext : & 'static [ u8 ] = "pNext must be NULL or a pointer to a valid instance of VkDeviceQueueGlobalPriorityCreateInfoEXT" . as_bytes ( ) ;
        pub const VUID_VkDeviceQueueCreateInfo_flags_parameter: &'static [u8] =
            "flags must be a valid combination of VkDeviceQueueCreateFlagBits values".as_bytes();
        pub const VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: &'static [u8] =
            "pQueuePriorities must be a valid pointer to an array of queueCount float values"
                .as_bytes();
        pub const VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: &'static [u8] =
            "queueCount must be greater than 0".as_bytes();
    );
}

pub struct DeviceQueueCreateInfoArray<'a, S: EnumeratorStorage<DeviceQueueCreateInfo<'a>>> {
    infos: S::InitStorage,
    _a: PhantomData<&'a ()>,
}

impl<'a, S: EnumeratorStorage<DeviceQueueCreateInfo<'a>>> fmt::Debug for DeviceQueueCreateInfoArray<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.infos.as_ref().iter()).finish()
    }
}

impl<'a, S: EnumeratorStorage<DeviceQueueCreateInfo<'a>>> std::ops::Deref for DeviceQueueCreateInfoArray<'a, S> {
    type Target = [DeviceQueueCreateInfo<'a>];

    fn deref(&self) -> &Self::Target {
        &self.infos.as_ref()
    }
}

impl<'a, S: EnumeratorStorage<DeviceQueueCreateInfo<'a>>> DeviceQueueCreateInfoArray<'a, S> {
    pub(crate) fn new(infos: S::InitStorage) -> Self {
        Self {
            infos,
            _a: PhantomData,
        }
    }
}

pub struct QueuePriorities<A> {
    priorities: A,
}

impl<A: AsRef<[f32]>> QueuePriorities<A> {
    pub fn new(priorities: A) -> Self {
        for p in priorities.as_ref().iter().copied() {
            assert!(0.0 <= p && p <= 1.0)
        }
        unsafe { Self::new_unchecked(priorities) }
    }
    pub unsafe fn new_unchecked(priorities: A) -> Self {
        Self { priorities }
    }
    pub fn with_num_queues(&self, num_queues: usize) -> QueuePriorities<&[f32]> {
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
pub struct DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage> {
    family_index: u32,
    to_write: &'initializer mut crate::enumerator_storage::UninitArrayInitializer<'storage, DeviceQueueCreateInfo<'params>>,
    pub family_properties: &'properties QueueFamilyProperties,
}

impl<'params, 'properties, 'initializer, 'storage> DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage> {
    /// internal only method to be called from [QueueFamilies::create_info_builder_iter]
    /// should pass the current queue_family_index, and set queue_count to max possible
    pub(crate) fn new(
        family_index: u32,
        to_write: &'initializer mut crate::enumerator_storage::UninitArrayInitializer<'storage, DeviceQueueCreateInfo<'params>>,
        family_properties: &'properties QueueFamilyProperties) -> Self
    {
        Self {
            family_index,
            to_write,
            family_properties,
        }
    }
    /// configure DeviceQueueCreateInfo for this family to use priorities.len queues with the given priorities, and with given flag
    pub fn push_config<A: AsRef<[f32]> + 'params>(
        self,
        priorities: &QueuePriorities<A>,
        flags: Option<impl vk::DeviceQueueCreateFlagsConst>
    ) -> crate::enumerator_storage::InitResult {
        assert!(priorities.len() <= self.family_properties.queue_count as usize);
        let info = DeviceQueueCreateInfo {
            inner: vk::DeviceQueueCreateInfo {
                s_type: vk::structure_type::DEVICE_QUEUE_CREATE_INFO.as_enum(),
                flags: flags.map_or(unsafe{vk::DeviceQueueCreateFlags::empty()}, |f|f.bitmask()),
                p_next: std::ptr::null(),
                queue_family_index: self.family_index,
                queue_count: priorities.len() as u32, // the assert already confirms no overflow from conversion
                p_queue_priorities:priorities.as_ptr(),
            },
            _refs: PhantomData
        };
        self.to_write.push(info)
    }
    /// like [configure], but will configure two [DeviceQueueCreateInfo] where one must be protected, and the other not
    /// based on rules in:
    /// VUID-VkDeviceCreateInfo-queueFamilyIndex-02802
    /// VUID-VkDeviceCreateInfo-pQueueCreateInfos-06755
    pub fn push_config_with_protected<A: AsRef<[f32]> + 'params>(
        self,
        priorities_for_non_protected: &QueuePriorities<A>,
        flags_for_non_protected: Option<impl vk::DeviceQueueCreateFlagsConst>,
        priorities_for_protected: &QueuePriorities<A>,
        flags_for_protected: Option<impl vk::DeviceQueueCreateFlagsConst>
    ) -> crate::enumerator_storage::InitResult {
        if let Some(flags_for_non_protected) = flags_for_non_protected {
            MUST_NOT_USE_PROTECTED_BIT::verify(flags_for_non_protected);
        }
        if let Some(flags_for_protected) = flags_for_protected {
            MUST_USE_PROTECTED_BIT::verify(flags_for_protected);
        }
        assert!(self.family_properties.queue_flags.contains(bitmask!(vk::queue_flag_bits: PROTECTED_BIT).bitmask()));
        assert!(priorities_for_non_protected.len() + priorities_for_protected.len() <= self.family_properties.queue_count as usize);

        let non_protected_info = DeviceQueueCreateInfo {
            inner: vk::DeviceQueueCreateInfo {
                s_type: vk::structure_type::DEVICE_QUEUE_CREATE_INFO.as_enum(),
                flags: flags_for_non_protected.map_or(unsafe{vk::DeviceQueueCreateFlags::empty()}, |f|f.bitmask()),
                p_next: std::ptr::null(),
                queue_family_index: self.family_index,
                queue_count: priorities_for_non_protected.len() as u32, // the assert already confirms no overflow from conversion
                p_queue_priorities:priorities_for_non_protected.as_ptr(),
            },
            _refs: PhantomData
        };
        self.to_write.push(non_protected_info)?;

        let protected_info = DeviceQueueCreateInfo {
            inner: vk::DeviceQueueCreateInfo {
                s_type: vk::structure_type::DEVICE_QUEUE_CREATE_INFO.as_enum(),
                flags: flags_for_protected.map_or(unsafe{vk::DeviceQueueCreateFlags::empty()}, |f|f.bitmask()),
                p_next: std::ptr::null(),
                queue_family_index: self.family_index,
                queue_count: priorities_for_protected.len() as u32, // the assert already confirms no overflow from conversion
                p_queue_priorities:priorities_for_protected.as_ptr(),
            },
            _refs: PhantomData
        };
        self.to_write.push(protected_info)
    }
}

verify_params!(MUST_NOT_USE_PROTECTED_BIT(Flags: vk::DeviceQueueCreateFlagsConst) {
    let flags = vk::raw_bitmask_from_type!(Flags);
    assert!(!flags.contains(vk::device_queue_create_flag_bits::PROTECTED_BIT), "flags_for_non_protected should not include PROTECTED_BIT");
});

verify_params!(MUST_USE_PROTECTED_BIT(Flags: vk::DeviceQueueCreateFlagsConst) {
    let flags = vk::raw_bitmask_from_type!(Flags);
    assert!(flags.contains(vk::device_queue_create_flag_bits::PROTECTED_BIT), "flags_for_protected must include PROTECTED_BIT");
});