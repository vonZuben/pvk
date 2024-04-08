use super::get_physical_device_queue_family_properties::{QueueFamiliesRef, QueueFamily};
use super::*;
use crate::device_type::{Config, DeviceType};
use crate::error::Error;
use crate::instance_type::Instance;
use crate::type_conversions::TransmuteSlice;
use vk_safe_sys as vk;

use crate::type_conversions::transmute_slice;

use std::fmt;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use vk::context::{Commands, Extensions, InstanceDependencies, LoadCommands};
use vk::has_command::{CreateDevice, DestroyDevice, EnumerateDeviceExtensionProperties};
use vk::Version;

pub trait VersionCheck<I> {
    const VALID: ();
}

impl<I: Version, D: Version> VersionCheck<I> for D {
    const VALID: () = {
        if D::VERSION.raw() > I::VERSION.raw() {
            panic!("version of Instance must be >= version of Device")
        }
    };
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateDevice.html
*/
impl<S: PhysicalDevice, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn create_device<'a, C, O>(
        &self,
        create_info: &DeviceCreateInfo<'a, C, S>,
        queue_properties: &'a QueueFamiliesRef<S>,
    ) -> Result<DeviceType<Config<'a, C, S>>, Error>
    where
        I::Commands: CreateDevice + Version,
        C: Commands + InstanceDependencies<I::Commands, O>,
        C::Commands: DestroyDevice + LoadCommands + Version + VersionCheck<I::Commands>,
        I::Commands: EnumerateDeviceExtensionProperties, // This is meant to be for a temporary safety check. Should be removed.
    {
        // check version requirement
        let _ = C::Commands::VALID;

        check_vuids::check_vuids!(CreateDevice);

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_ppEnabledExtensionNames_01387: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "All required device extensions for each extension in the VkDeviceCreateInfo::ppEnabledExtensionNames"
            "list must also be present in that list"
            }

            // This is ensured by the context creation macros
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // ensured by PhysicalDevice creation
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_pCreateInfo_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pCreateInfo must be a valid pointer to a valid VkDeviceCreateInfo structure"
            }

            // ensured by DeviceCreateInfo creation
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // AllocationCallbacks not supported
        }

        #[allow(unused_labels)]
        'VUID_vkCreateDevice_pDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pDevice must be a valid pointer to a VkDevice handle"
            }

            // MaybeUninit
        }

        let mut device = MaybeUninit::uninit();

        // *********************************************
        // *********Fix with extension support**********
        // **VUID_VkDeviceCreateInfo_pProperties_04451**
        // *********************************************
        for e in self.enumerate_device_extension_properties(None, Vec::new())? {
            if e.extension_name() == "VK_KHR_portability_subset" {
                panic!("Physical device with VK_KHR_portability_subset is not supported")
            }
        }
        // *********************************************
        unsafe {
            let res = self.instance.commands.CreateDevice().get_fptr()(
                self.handle,
                &create_info.inner,
                std::ptr::null(),
                device.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(DeviceType::load_commands(
                device.assume_init(),
                Config::new(
                    std::slice::from_raw_parts(
                        create_info.inner.p_queue_create_infos,
                        create_info.inner.queue_create_info_count.try_into()?,
                    )
                    .safe_transmute_slice(),
                    queue_properties,
                ),
            )?)
        }
    }
}

//===========InstanceCreateInfo
pub struct DeviceCreateInfo<'a, C, S> {
    pub(crate) inner: vk::DeviceCreateInfo,
    _config: PhantomData<C>,
    _refs: PhantomData<&'a ()>,
    _scope: PhantomData<S>,
}

impl<'a> DeviceCreateInfo<'a, (), ()> {
    pub fn new<C: Copy + Extensions + Commands, S>(
        _: C,
        queue_create_info: &'a [DeviceQueueCreateInfo<S>],
    ) -> DeviceCreateInfo<'a, C, S>
    where
        C::Commands: Version,
    {
        check_vuids::check_vuids!(DeviceCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_queueFamilyIndex_02802: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "The queueFamilyIndex member of each element of pQueueCreateInfos must be unique within"
            "pQueueCreateInfos , except that two members can share the same queueFamilyIndex if"
            "one describes protected-capable queues and one describes queues that are not protected-capable"
            }

            // DeviceQueueCreateInfo is created by consuming QueueFamily(with unique index)
            // each one can only be consumed once within the queue config scope
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pQueueCreateInfos_06755: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If multiple elements of pQueueCreateInfos share the same queueFamilyIndex, the sum"
            "of their queueCount members must be less than or equal to the queueCount member of"
            "the VkQueueFamilyProperties structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties"
            "in the pQueueFamilyProperties[queueFamilyIndex]"
            }

            // this is currently not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pQueueCreateInfos_06654: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If multiple elements of pQueueCreateInfos share the same queueFamilyIndex, then all"
            "of such elements must have the same global priority level, which can be specified"
            "explicitly by the including a VkDeviceQueueGlobalPriorityCreateInfoKHR structure in"
            "the pNext chain, or by the implicit default value"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_00373: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkPhysicalDeviceFeatures2 structure, then pEnabledFeatures"
            "must be NULL"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_01840: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkPhysicalDeviceProperties::apiVersion advertises Vulkan 1.1 or later, ppEnabledExtensionNames"
            "must not contain VK_AMD_negative_viewport_height"
            }

            if C::Commands::VERSION >= crate::VkVersion::new(1, 1, 0) {
                for e in C::list_of_extensions().as_ref().iter().copied() {
                    if crate::vk_str!("VK_AMD_negative_viewport_height") == e {
                        panic!("violated VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_01840")
                    }
                }
            }
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_00374: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "ppEnabledExtensionNames must not contain both VK_KHR_maintenance1 and VK_AMD_negative_viewport_height"
            }

            #[allow(non_snake_case)]
            let mut VK_KHR_maintenance1 = false;
            #[allow(non_snake_case)]
            let mut VK_AMD_negative_viewport_height = false;

            for e in C::list_of_extensions().as_ref().iter().copied() {
                if crate::vk_str!("VK_KHR_maintenance1") == e {
                    VK_KHR_maintenance1 = true;
                }

                if crate::vk_str!("VK_AMD_negative_viewport_height") == e {
                    VK_AMD_negative_viewport_height = true;
                }
            }

            if VK_AMD_negative_viewport_height && VK_KHR_maintenance1 {
                panic!("violated VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_00374")
            }
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_03328: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "ppEnabledExtensionNames must not contain both VK_KHR_buffer_device_address and VK_EXT_buffer_device_address"
            }

            #[allow(non_snake_case)]
            let mut VK_KHR_buffer_device_address = false;
            #[allow(non_snake_case)]
            let mut VK_EXT_buffer_device_address = false;

            for e in C::list_of_extensions().as_ref().iter().copied() {
                if crate::vk_str!("VK_KHR_buffer_device_address") == e {
                    VK_KHR_buffer_device_address = true;
                }

                if crate::vk_str!("VK_EXT_buffer_device_address") == e {
                    VK_EXT_buffer_device_address = true;
                }
            }

            if VK_KHR_buffer_device_address && VK_EXT_buffer_device_address {
                panic!("violated VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_03328")
            }
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_04748: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkPhysicalDeviceVulkan12Features structure and VkPhysicalDeviceVulkan12Features::bufferDeviceAddress"
            "is VK_TRUE, ppEnabledExtensionNames must not contain VK_EXT_buffer_device_address"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_02829: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkPhysicalDeviceVulkan11Features structure, then it"
            "must not include a VkPhysicalDevice16BitStorageFeatures, VkPhysicalDeviceMultiviewFeatures,"
            "VkPhysicalDeviceVariablePointersFeatures, VkPhysicalDeviceProtectedMemoryFeatures,"
            "VkPhysicalDeviceSamplerYcbcrConversionFeatures, or VkPhysicalDeviceShaderDrawParametersFeatures"
            "structure"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_02830: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkPhysicalDeviceVulkan12Features structure, then it"
            "must not include a VkPhysicalDevice8BitStorageFeatures, VkPhysicalDeviceShaderAtomicInt64Features,"
            "VkPhysicalDeviceShaderFloat16Int8Features, VkPhysicalDeviceDescriptorIndexingFeatures,"
            "VkPhysicalDeviceScalarBlockLayoutFeatures, VkPhysicalDeviceImagelessFramebufferFeatures,"
            "VkPhysicalDeviceUniformBufferStandardLayoutFeatures, VkPhysicalDeviceShaderSubgroupExtendedTypesFeatures,"
            "VkPhysicalDeviceSeparateDepthStencilLayoutsFeatures, VkPhysicalDeviceHostQueryResetFeatures,"
            "VkPhysicalDeviceTimelineSemaphoreFeatures, VkPhysicalDeviceBufferDeviceAddressFeatures,"
            "or VkPhysicalDeviceVulkanMemoryModelFeatures structure"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_04476: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If ppEnabledExtensionNames contains \"VK_KHR_shader_draw_parameters\" and the pNext"
            "chain includes a VkPhysicalDeviceVulkan11Features structure, then VkPhysicalDeviceVulkan11Features::shaderDrawParameters"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02831: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If ppEnabledExtensionNames contains \"VK_KHR_draw_indirect_count\" and the pNext chain"
            "includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::drawIndirectCount"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02832: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If ppEnabledExtensionNames contains \"VK_KHR_sampler_mirror_clamp_to_edge\" and the"
            "pNext chain includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::samplerMirrorClampToEdge"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02833: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If ppEnabledExtensionNames contains \"VK_EXT_descriptor_indexing\" and the pNext chain"
            "includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::descriptorIndexing"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02834: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If ppEnabledExtensionNames contains \"VK_EXT_sampler_filter_minmax\" and the pNext"
            "chain includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::samplerFilterMinmax"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02835: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If ppEnabledExtensionNames contains \"VK_EXT_shader_viewport_index_layer\" and the"
            "pNext chain includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::shaderOutputViewportIndex"
            "and VkPhysicalDeviceVulkan12Features::shaderOutputLayer must both be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_06532: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkPhysicalDeviceVulkan13Features structure, then it"
            "must not include a VkPhysicalDeviceDynamicRenderingFeatures, VkPhysicalDeviceImageRobustnessFeatures,"
            "VkPhysicalDeviceInlineUniformBlockFeatures, VkPhysicalDeviceMaintenance4Features,"
            "VkPhysicalDevicePipelineCreationCacheControlFeatures, VkPhysicalDevicePrivateDataFeatures,"
            "VkPhysicalDeviceShaderDemoteToHelperInvocationFeatures, VkPhysicalDeviceShaderIntegerDotProductFeatures,"
            "VkPhysicalDeviceShaderTerminateInvocationFeatures, VkPhysicalDeviceSubgroupSizeControlFeatures,"
            "VkPhysicalDeviceSynchronization2Features, VkPhysicalDeviceTextureCompressionASTCHDRFeatures,"
            "or VkPhysicalDeviceZeroInitializeWorkgroupMemoryFeatures structure"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pProperties_04451: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the VK_KHR_portability_subset extension is included in pProperties of vkEnumerateDeviceExtensionProperties,"
            "ppEnabledExtensionNames must include \"VK_KHR_portability_subset\""
            }

            // TODO: this is definitely bad
            // *********************************************
            // when extensions are properly supported, do better checking
            // for now, just check if a VK_KHR_portability_subset device is being used and panic
            // this check is performed in create_device for now
            // *********************************************
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_shadingRateImage_04478: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the shadingRateImage feature is enabled, the pipelineFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_shadingRateImage_04479: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the shadingRateImage feature is enabled, the primitiveFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_shadingRateImage_04480: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the shadingRateImage feature is enabled, the attachmentFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_fragmentDensityMap_04481: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the fragmentDensityMap feature is enabled, the pipelineFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_fragmentDensityMap_04482: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the fragmentDensityMap feature is enabled, the primitiveFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_fragmentDensityMap_04483: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the fragmentDensityMap feature is enabled, the attachmentFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_04896: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If sparseImageInt64Atomics is enabled, shaderImageInt64Atomics must be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_04897: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If sparseImageFloat32Atomics is enabled, shaderImageFloat32Atomics must be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_04898: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If sparseImageFloat32AtomicAdd is enabled, shaderImageFloat32AtomicAdd must be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_sparseImageFloat32AtomicMinMax_04975: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If sparseImageFloat32AtomicMinMax is enabled, shaderImageFloat32AtomicMinMax must"
            "be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_08095: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If descriptorBuffer is enabled, ppEnabledExtensionNames must not contain VK_AMD_shader_fragment_mask"
            }

            // TODO: features and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "Each pNext member of any structure (including this one) in the pNext chain must be"
            "either NULL or a pointer to a valid instance of VkDeviceDeviceMemoryReportCreateInfoEXT,"
            "VkDeviceDiagnosticsConfigCreateInfoNV, VkDeviceGroupDeviceCreateInfo, VkDeviceMemoryOverallocationCreateInfoAMD,"
            "VkDevicePrivateDataCreateInfo, VkPhysicalDevice16BitStorageFeatures, VkPhysicalDevice4444FormatsFeaturesEXT,"
            "VkPhysicalDevice8BitStorageFeatures, VkPhysicalDeviceASTCDecodeFeaturesEXT, VkPhysicalDeviceAccelerationStructureFeaturesKHR,"
            "VkPhysicalDeviceAddressBindingReportFeaturesEXT, VkPhysicalDeviceAmigoProfilingFeaturesSEC,"
            "VkPhysicalDeviceAttachmentFeedbackLoopDynamicStateFeaturesEXT, VkPhysicalDeviceAttachmentFeedbackLoopLayoutFeaturesEXT,"
            "VkPhysicalDeviceBlendOperationAdvancedFeaturesEXT, VkPhysicalDeviceBorderColorSwizzleFeaturesEXT,"
            "VkPhysicalDeviceBufferDeviceAddressFeatures, VkPhysicalDeviceBufferDeviceAddressFeaturesEXT,"
            "VkPhysicalDeviceClusterCullingShaderFeaturesHUAWEI, VkPhysicalDeviceCoherentMemoryFeaturesAMD,"
            "VkPhysicalDeviceColorWriteEnableFeaturesEXT, VkPhysicalDeviceComputeShaderDerivativesFeaturesNV,"
            "VkPhysicalDeviceConditionalRenderingFeaturesEXT, VkPhysicalDeviceCooperativeMatrixFeaturesKHR,"
            "VkPhysicalDeviceCooperativeMatrixFeaturesNV, VkPhysicalDeviceCopyMemoryIndirectFeaturesNV,"
            "VkPhysicalDeviceCornerSampledImageFeaturesNV, VkPhysicalDeviceCoverageReductionModeFeaturesNV,"
            "VkPhysicalDeviceCubicClampFeaturesQCOM, VkPhysicalDeviceCubicWeightsFeaturesQCOM,"
            "VkPhysicalDeviceCustomBorderColorFeaturesEXT, VkPhysicalDeviceDedicatedAllocationImageAliasingFeaturesNV,"
            "VkPhysicalDeviceDepthBiasControlFeaturesEXT, VkPhysicalDeviceDepthClampZeroOneFeaturesEXT,"
            "VkPhysicalDeviceDepthClipControlFeaturesEXT, VkPhysicalDeviceDepthClipEnableFeaturesEXT,"
            "VkPhysicalDeviceDescriptorBufferFeaturesEXT, VkPhysicalDeviceDescriptorIndexingFeatures,"
            "VkPhysicalDeviceDescriptorPoolOverallocationFeaturesNV, VkPhysicalDeviceDescriptorSetHostMappingFeaturesVALVE,"
            "VkPhysicalDeviceDeviceGeneratedCommandsComputeFeaturesNV, VkPhysicalDeviceDeviceGeneratedCommandsFeaturesNV,"
            "VkPhysicalDeviceDeviceMemoryReportFeaturesEXT, VkPhysicalDeviceDiagnosticsConfigFeaturesNV,"
            "VkPhysicalDeviceDisplacementMicromapFeaturesNV, VkPhysicalDeviceDynamicRenderingFeatures,"
            "VkPhysicalDeviceDynamicRenderingUnusedAttachmentsFeaturesEXT, VkPhysicalDeviceExclusiveScissorFeaturesNV,"
            "VkPhysicalDeviceExtendedDynamicState2FeaturesEXT, VkPhysicalDeviceExtendedDynamicState3FeaturesEXT,"
            "VkPhysicalDeviceExtendedDynamicStateFeaturesEXT, VkPhysicalDeviceExtendedSparseAddressSpaceFeaturesNV,"
            "VkPhysicalDeviceExternalFormatResolveFeaturesANDROID, VkPhysicalDeviceExternalMemoryRDMAFeaturesNV,"
            "VkPhysicalDeviceExternalMemoryScreenBufferFeaturesQNX, VkPhysicalDeviceFaultFeaturesEXT,"
            "VkPhysicalDeviceFeatures2, VkPhysicalDeviceFragmentDensityMap2FeaturesEXT, VkPhysicalDeviceFragmentDensityMapFeaturesEXT,"
            "VkPhysicalDeviceFragmentDensityMapOffsetFeaturesQCOM, VkPhysicalDeviceFragmentShaderBarycentricFeaturesKHR,"
            "VkPhysicalDeviceFragmentShaderInterlockFeaturesEXT, VkPhysicalDeviceFragmentShadingRateEnumsFeaturesNV,"
            "VkPhysicalDeviceFragmentShadingRateFeaturesKHR, VkPhysicalDeviceFrameBoundaryFeaturesEXT,"
            "VkPhysicalDeviceGlobalPriorityQueryFeaturesKHR, VkPhysicalDeviceGraphicsPipelineLibraryFeaturesEXT,"
            "VkPhysicalDeviceHostImageCopyFeaturesEXT, VkPhysicalDeviceHostQueryResetFeatures,"
            "VkPhysicalDeviceImage2DViewOf3DFeaturesEXT, VkPhysicalDeviceImageCompressionControlFeaturesEXT,"
            "VkPhysicalDeviceImageCompressionControlSwapchainFeaturesEXT, VkPhysicalDeviceImageProcessing2FeaturesQCOM,"
            "VkPhysicalDeviceImageProcessingFeaturesQCOM, VkPhysicalDeviceImageRobustnessFeatures,"
            "VkPhysicalDeviceImageSlicedViewOf3DFeaturesEXT, VkPhysicalDeviceImageViewMinLodFeaturesEXT,"
            "VkPhysicalDeviceImagelessFramebufferFeatures, VkPhysicalDeviceIndexTypeUint8FeaturesEXT,"
            "VkPhysicalDeviceInheritedViewportScissorFeaturesNV, VkPhysicalDeviceInlineUniformBlockFeatures,"
            "VkPhysicalDeviceInvocationMaskFeaturesHUAWEI, VkPhysicalDeviceLegacyDitheringFeaturesEXT,"
            "VkPhysicalDeviceLineRasterizationFeaturesEXT, VkPhysicalDeviceLinearColorAttachmentFeaturesNV,"
            "VkPhysicalDeviceMaintenance4Features, VkPhysicalDeviceMaintenance5FeaturesKHR, VkPhysicalDeviceMemoryDecompressionFeaturesNV,"
            "VkPhysicalDeviceMemoryPriorityFeaturesEXT, VkPhysicalDeviceMeshShaderFeaturesEXT,"
            "VkPhysicalDeviceMeshShaderFeaturesNV, VkPhysicalDeviceMultiDrawFeaturesEXT, VkPhysicalDeviceMultisampledRenderToSingleSampledFeaturesEXT,"
            "VkPhysicalDeviceMultiviewFeatures, VkPhysicalDeviceMultiviewPerViewRenderAreasFeaturesQCOM,"
            "VkPhysicalDeviceMultiviewPerViewViewportsFeaturesQCOM, VkPhysicalDeviceMutableDescriptorTypeFeaturesEXT,"
            "VkPhysicalDeviceNestedCommandBufferFeaturesEXT, VkPhysicalDeviceNonSeamlessCubeMapFeaturesEXT,"
            "VkPhysicalDeviceOpacityMicromapFeaturesEXT, VkPhysicalDeviceOpticalFlowFeaturesNV,"
            "VkPhysicalDevicePageableDeviceLocalMemoryFeaturesEXT, VkPhysicalDevicePerformanceQueryFeaturesKHR,"
            "VkPhysicalDevicePipelineCreationCacheControlFeatures, VkPhysicalDevicePipelineExecutablePropertiesFeaturesKHR,"
            "VkPhysicalDevicePipelineLibraryGroupHandlesFeaturesEXT, VkPhysicalDevicePipelinePropertiesFeaturesEXT,"
            "VkPhysicalDevicePipelineProtectedAccessFeaturesEXT, VkPhysicalDevicePipelineRobustnessFeaturesEXT,"
            "VkPhysicalDevicePortabilitySubsetFeaturesKHR, VkPhysicalDevicePresentBarrierFeaturesNV,"
            "VkPhysicalDevicePresentIdFeaturesKHR, VkPhysicalDevicePresentWaitFeaturesKHR, VkPhysicalDevicePrimitiveTopologyListRestartFeaturesEXT,"
            "VkPhysicalDevicePrimitivesGeneratedQueryFeaturesEXT, VkPhysicalDevicePrivateDataFeatures,"
            "VkPhysicalDeviceProtectedMemoryFeatures, VkPhysicalDeviceProvokingVertexFeaturesEXT,"
            "VkPhysicalDeviceRGBA10X6FormatsFeaturesEXT, VkPhysicalDeviceRasterizationOrderAttachmentAccessFeaturesEXT,"
            "VkPhysicalDeviceRayQueryFeaturesKHR, VkPhysicalDeviceRayTracingInvocationReorderFeaturesNV,"
            "VkPhysicalDeviceRayTracingMaintenance1FeaturesKHR, VkPhysicalDeviceRayTracingMotionBlurFeaturesNV,"
            "VkPhysicalDeviceRayTracingPipelineFeaturesKHR, VkPhysicalDeviceRayTracingPositionFetchFeaturesKHR,"
            "VkPhysicalDeviceRepresentativeFragmentTestFeaturesNV, VkPhysicalDeviceRobustness2FeaturesEXT,"
            "VkPhysicalDeviceSamplerYcbcrConversionFeatures, VkPhysicalDeviceScalarBlockLayoutFeatures,"
            "VkPhysicalDeviceSeparateDepthStencilLayoutsFeatures, VkPhysicalDeviceShaderAtomicFloat2FeaturesEXT,"
            "VkPhysicalDeviceShaderAtomicFloatFeaturesEXT, VkPhysicalDeviceShaderAtomicInt64Features,"
            "VkPhysicalDeviceShaderClockFeaturesKHR, VkPhysicalDeviceShaderCoreBuiltinsFeaturesARM,"
            "VkPhysicalDeviceShaderDemoteToHelperInvocationFeatures, VkPhysicalDeviceShaderDrawParametersFeatures,"
            "VkPhysicalDeviceShaderEarlyAndLateFragmentTestsFeaturesAMD, VkPhysicalDeviceShaderEnqueueFeaturesAMDX,"
            "VkPhysicalDeviceShaderFloat16Int8Features, VkPhysicalDeviceShaderImageAtomicInt64FeaturesEXT,"
            "VkPhysicalDeviceShaderImageFootprintFeaturesNV, VkPhysicalDeviceShaderIntegerDotProductFeatures,"
            "VkPhysicalDeviceShaderIntegerFunctions2FeaturesINTEL, VkPhysicalDeviceShaderModuleIdentifierFeaturesEXT,"
            "VkPhysicalDeviceShaderObjectFeaturesEXT, VkPhysicalDeviceShaderSMBuiltinsFeaturesNV,"
            "VkPhysicalDeviceShaderSubgroupExtendedTypesFeatures, VkPhysicalDeviceShaderSubgroupUniformControlFlowFeaturesKHR,"
            "VkPhysicalDeviceShaderTerminateInvocationFeatures, VkPhysicalDeviceShaderTileImageFeaturesEXT,"
            "VkPhysicalDeviceShadingRateImageFeaturesNV, VkPhysicalDeviceSubgroupSizeControlFeatures,"
            "VkPhysicalDeviceSubpassMergeFeedbackFeaturesEXT, VkPhysicalDeviceSubpassShadingFeaturesHUAWEI,"
            "VkPhysicalDeviceSwapchainMaintenance1FeaturesEXT, VkPhysicalDeviceSynchronization2Features,"
            "VkPhysicalDeviceTexelBufferAlignmentFeaturesEXT, VkPhysicalDeviceTextureCompressionASTCHDRFeatures,"
            "VkPhysicalDeviceTilePropertiesFeaturesQCOM, VkPhysicalDeviceTimelineSemaphoreFeatures,"
            "VkPhysicalDeviceTransformFeedbackFeaturesEXT, VkPhysicalDeviceUniformBufferStandardLayoutFeatures,"
            "VkPhysicalDeviceVariablePointersFeatures, VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT,"
            "VkPhysicalDeviceVertexInputDynamicStateFeaturesEXT, VkPhysicalDeviceVulkan11Features,"
            "VkPhysicalDeviceVulkan12Features, VkPhysicalDeviceVulkan13Features, VkPhysicalDeviceVulkanMemoryModelFeatures,"
            "VkPhysicalDeviceWorkgroupMemoryExplicitLayoutFeaturesKHR, VkPhysicalDeviceYcbcr2Plane444FormatsFeaturesEXT,"
            "VkPhysicalDeviceYcbcrDegammaFeaturesQCOM, VkPhysicalDeviceYcbcrImageArraysFeaturesEXT,"
            "or VkPhysicalDeviceZeroInitializeWorkgroupMemoryFeatures"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_sType_unique: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "The sType value of each struct in the pNext chain must be unique, with the exception"
            "of structures of type VkDeviceDeviceMemoryReportCreateInfoEXT or VkDevicePrivateDataCreateInfo"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_flags_zerobitmask: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be 0"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pQueueCreateInfos_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pQueueCreateInfos must be a valid pointer to an array of queueCreateInfoCount valid"
            "VkDeviceQueueCreateInfo structures"
            }

            // rust reference; VkDeviceQueueCreateInfo self validated
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledLayerNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If enabledLayerCount is not 0, ppEnabledLayerNames must be a valid pointer to an array"
            "of enabledLayerCount null-terminated UTF-8 strings"
            }

            // TODO: layers not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If enabledExtensionCount is not 0, ppEnabledExtensionNames must be a valid pointer"
            "to an array of enabledExtensionCount null-terminated UTF-8 strings"
            }

            // a proper implementation of the unsafe Extensions trait ensures this
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pEnabledFeatures_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pEnabledFeatures is not NULL, pEnabledFeatures must be a valid pointer to a valid"
            "VkPhysicalDeviceFeatures structure"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_queueCreateInfoCount_arraylength: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueCreateInfoCount must be greater than 0"
            }

            // checked in DeviceQueueCreateInfoConfiguration, but maybe I should change how this works
        }

        let extensions = C::list_of_extensions();
        let extensions = extensions.as_ref();

        DeviceCreateInfo {
            inner: vk::DeviceCreateInfo {
                s_type: vk::StructureType::DEVICE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceCreateFlags::empty(),
                queue_create_info_count: queue_create_info.len() as u32,
                p_queue_create_infos: transmute_slice(queue_create_info).as_ptr(),
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: extensions
                    .len()
                    .try_into()
                    .expect("list of extensions len bigger than u32::MAX"),
                pp_enabled_extension_names: extensions.as_ptr().cast(),
                p_enabled_features: std::ptr::null(),
            },
            _config: PhantomData,
            _refs: PhantomData,
            _scope: PhantomData,
        }
    }
}

input_struct_wrapper!(DeviceQueueCreateInfo);

impl<S> fmt::Debug for DeviceQueueCreateInfo<'_, S> {
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

impl<'a, S> DeviceQueueCreateInfo<'a, S> {
    array!(queue_priorities, p_queue_priorities, queue_count, f32);

    pub fn new<Q>(
        priorities: QueuePriorities<'a>,
        family: QueueFamily<(S, Q)>,
    ) -> Result<Self, Error> {
        check_vuids::check_vuids!(DeviceQueueCreateInfo);

        let priorities_len: u32 = priorities.len().try_into()?;

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueFamilyIndex must be less than pQueueFamilyPropertyCount returned by vkGetPhysicalDeviceQueueFamilyProperties"
            }

            // QueueFamily invariant
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_00382: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueCount must be less than or equal to the queueCount member of the VkQueueFamilyProperties"
            "structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties in the pQueueFamilyProperties[queueFamilyIndex]"
            }

            assert!(priorities_len <= family.queue_count)
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "Each element of pQueuePriorities must be between 0.0 and 1.0 inclusive"
            }

            // [QueuePriorities]
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_02861: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the protectedMemory feature is not enabled, the VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT"
            "bit of flags must not be set"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_06449: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If flags includes VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT, queueFamilyIndex must be the"
            "index of a queue family that includes the VK_QUEUE_PROTECTED_BIT capability"
            }

            // flags not supported for now
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pNext must be NULL or a pointer to a valid instance of VkDeviceQueueGlobalPriorityCreateInfoKHR"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_unique: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "The sType value of each struct in the pNext chain must be unique"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "flags must be a valid combination of VkDeviceQueueCreateFlagBits values"
            }

            // vk::DeviceQueueCreateFlags, and checking VUID_VkDeviceQueueCreateInfo_flags_06449 above
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pQueuePriorities must be a valid pointer to an array of queueCount float values"
            }

            // QueuePriorities
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "queueCount must be greater than 0"
            }

            // QueuePriorities
        }

        Ok(Self {
            inner: vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: family.family_index,
                queue_count: priorities_len,
                p_queue_priorities: priorities.as_ptr(),
            },
            _params: PhantomData,
            _scope: PhantomData,
        })
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
    pub fn with_num_queues(&self, num_queues: u32) -> Self {
        assert!(num_queues > 0);
        let num_queues: usize = num_queues.try_into().unwrap();
        unsafe { QueuePriorities::new_unchecked(&self.priorities.as_ref()[..num_queues]) }
    }
    pub fn len(&self) -> usize {
        self.priorities.as_ref().len()
    }
    pub fn as_ptr(&self) -> *const f32 {
        self.priorities.as_ref().as_ptr()
    }
}
