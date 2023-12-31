use super::*;
use crate::device::{Config, DeviceType};
use crate::error::Error;
use crate::instance::Instance;
use vk_safe_sys as vk;

use crate::type_conversions::{transmute_slice, SafeTransmute};

use std::fmt;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use vk::commands::{Commands, LoadCommands, Version};
use vk::has_command::{CreateDevice, DestroyDevice, EnumerateDeviceExtensionProperties};

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateDevice.html
*/
impl<S: PhysicalDevice, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn create_device<C>(
        &self,
        create_info: &DeviceCreateInfo<C, S>,
    ) -> Result<DeviceType<Config<C, S>>, Error>
    where
        I::Commands: CreateDevice + EnumerateDeviceExtensionProperties,
        C: Commands,
        C::Commands: DestroyDevice + LoadCommands + Version,
    {
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
            Ok(DeviceType::load_commands(device.assume_init())?)
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
    pub const fn new<C: Copy, S>(
        _: C,
        queue_create_info: &'a [DeviceQueueCreateInfo<S>],
    ) -> DeviceCreateInfo<'a, C, S> {
        check_vuids::check_vuids!(DeviceCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_queueFamilyIndex_02802: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "The queueFamilyIndex member of each element of pQueueCreateInfos must be unique within"
            "pQueueCreateInfos , except that two members can share the same queueFamilyIndex if"
            "one describes protected-capable queues and one describes queues that are not protected-capable"
            }

            // handled by DeviceQueueCreateInfoConfiguration
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pQueueCreateInfos_06755: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If multiple elements of pQueueCreateInfos share the same queueFamilyIndex, the sum"
            "of their queueCount members must be less than or equal to the queueCount member of"
            "the VkQueueFamilyProperties structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties"
            "in the pQueueFamilyProperties[queueFamilyIndex]"
            }

            // handled by DeviceQueueCreateInfoConfiguration
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pQueueCreateInfos_06654: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
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
            check_vuids::cur_description! {
            "If the pNext chain includes a VkPhysicalDeviceFeatures2 structure, then pEnabledFeatures"
            "must be NULL"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_01840: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If VkPhysicalDeviceProperties::apiVersion advertises Vulkan 1.1 or later, ppEnabledExtensionNames"
            "must not contain VK_AMD_negative_viewport_height"
            }

            // TODO: extensions not yet supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_00374: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "ppEnabledExtensionNames must not contain both VK_KHR_maintenance1 and VK_AMD_negative_viewport_height"
            }

            // TODO: extensions not yet supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_03328: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "ppEnabledExtensionNames must not contain both VK_KHR_buffer_device_address and VK_EXT_buffer_device_address"
            }

            // TODO: extensions not yet supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_04748: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the pNext chain includes a VkPhysicalDeviceVulkan12Features structure and VkPhysicalDeviceVulkan12Features::bufferDeviceAddress"
            "is VK_TRUE, ppEnabledExtensionNames must not contain VK_EXT_buffer_device_address"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_02829: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
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
            check_vuids::cur_description! {
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
            check_vuids::cur_description! {
            "If ppEnabledExtensionNames contains \"VK_KHR_shader_draw_parameters\" and the pNext"
            "chain includes a VkPhysicalDeviceVulkan11Features structure, then VkPhysicalDeviceVulkan11Features::shaderDrawParameters"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02831: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If ppEnabledExtensionNames contains \"VK_KHR_draw_indirect_count\" and the pNext chain"
            "includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::drawIndirectCount"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02832: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If ppEnabledExtensionNames contains \"VK_KHR_sampler_mirror_clamp_to_edge\" and the"
            "pNext chain includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::samplerMirrorClampToEdge"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02833: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If ppEnabledExtensionNames contains \"VK_EXT_descriptor_indexing\" and the pNext chain"
            "includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::descriptorIndexing"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02834: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If ppEnabledExtensionNames contains \"VK_EXT_sampler_filter_minmax\" and the pNext"
            "chain includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::samplerFilterMinmax"
            "must be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_02835: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If ppEnabledExtensionNames contains \"VK_EXT_shader_viewport_index_layer\" and the"
            "pNext chain includes a VkPhysicalDeviceVulkan12Features structure, then VkPhysicalDeviceVulkan12Features::shaderOutputViewportIndex"
            "and VkPhysicalDeviceVulkan12Features::shaderOutputLayer must both be VK_TRUE"
            }

            // TODO: p_next and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_06532: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
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
            check_vuids::cur_description! {
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
            check_vuids::cur_description! {
            "If the shadingRateImage feature is enabled, the pipelineFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_shadingRateImage_04479: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the shadingRateImage feature is enabled, the primitiveFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_shadingRateImage_04480: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the shadingRateImage feature is enabled, the attachmentFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_fragmentDensityMap_04481: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the fragmentDensityMap feature is enabled, the pipelineFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_fragmentDensityMap_04482: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the fragmentDensityMap feature is enabled, the primitiveFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_fragmentDensityMap_04483: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the fragmentDensityMap feature is enabled, the attachmentFragmentShadingRate feature"
            "must not be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_04896: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If sparseImageInt64Atomics is enabled, shaderImageInt64Atomics must be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_04897: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If sparseImageFloat32Atomics is enabled, shaderImageFloat32Atomics must be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_04898: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If sparseImageFloat32AtomicAdd is enabled, shaderImageFloat32AtomicAdd must be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_sparseImageFloat32AtomicMinMax_04975: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If sparseImageFloat32AtomicMinMax is enabled, shaderImageFloat32AtomicMinMax must"
            "be enabled"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_None_08095: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If descriptorBuffer is enabled, ppEnabledExtensionNames must not contain VK_AMD_shader_fragment_mask"
            }

            // TODO: features and extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "sType must be VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
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
            check_vuids::cur_description! {
            "The sType value of each struct in the pNext chain must be unique, with the exception"
            "of structures of type VkDeviceDeviceMemoryReportCreateInfoEXT or VkDevicePrivateDataCreateInfo"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_flags_zerobitmask: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "flags must be 0"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pQueueCreateInfos_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pQueueCreateInfos must be a valid pointer to an array of queueCreateInfoCount valid"
            "VkDeviceQueueCreateInfo structures"
            }

            // rust reference; VkDeviceQueueCreateInfo self validated
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledLayerNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If enabledLayerCount is not 0, ppEnabledLayerNames must be a valid pointer to an array"
            "of enabledLayerCount null-terminated UTF-8 strings"
            }

            // TODO: layers not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_ppEnabledExtensionNames_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If enabledExtensionCount is not 0, ppEnabledExtensionNames must be a valid pointer"
            "to an array of enabledExtensionCount null-terminated UTF-8 strings"
            }

            // TODO: extensions not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_pEnabledFeatures_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If pEnabledFeatures is not NULL, pEnabledFeatures must be a valid pointer to a valid"
            "VkPhysicalDeviceFeatures structure"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceCreateInfo_queueCreateInfoCount_arraylength: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "queueCreateInfoCount must be greater than 0"
            }

            // checked in DeviceQueueCreateInfoConfiguration, but maybe I should change how this works
        }

        DeviceCreateInfo {
            inner: vk::DeviceCreateInfo {
                s_type: vk::StructureType::DEVICE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceCreateFlags::empty(),
                queue_create_info_count: queue_create_info.len() as u32,
                p_queue_create_infos: transmute_slice(queue_create_info).as_ptr(),
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: 0,
                pp_enabled_extension_names: std::ptr::null(),
                p_enabled_features: std::ptr::null(),
            },
            _config: PhantomData,
            _refs: PhantomData,
            _scope: PhantomData,
        }
    }
}

/// A safe to use [vk::DeviceQueueCreateInfo]
#[repr(transparent)]
pub struct DeviceQueueCreateInfo<'a, S> {
    inner: vk::DeviceQueueCreateInfo,
    _refs: PhantomData<&'a ()>,
    _scope: PhantomData<S>,
}

unsafe impl<S> SafeTransmute<vk::DeviceQueueCreateInfo> for DeviceQueueCreateInfo<'_, S> {}

impl<S> DeviceQueueCreateInfo<'_, S> {
    array!(queue_priorities, p_queue_priorities, queue_count, f32);
}

impl<S> std::ops::Deref for DeviceQueueCreateInfo<'_, S> {
    type Target = vk::DeviceQueueCreateInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

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

pub struct DeviceQueueCreateInfoArray<'a, A: ArrayStorage<DeviceQueueCreateInfo<'a, S>>, S> {
    infos: A::InitStorage,
    _a: PhantomData<&'a ()>,
    _scope: PhantomData<S>,
}

impl<'a, A: ArrayStorage<DeviceQueueCreateInfo<'a, S>>, S> fmt::Debug
    for DeviceQueueCreateInfoArray<'a, A, S>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.infos.as_ref().iter()).finish()
    }
}

impl<'a, A: ArrayStorage<DeviceQueueCreateInfo<'a, S>>, S> std::ops::Deref
    for DeviceQueueCreateInfoArray<'a, A, S>
{
    type Target = [DeviceQueueCreateInfo<'a, S>];

    fn deref(&self) -> &Self::Target {
        &self.infos.as_ref()
    }
}

impl<'a, A: ArrayStorage<DeviceQueueCreateInfo<'a, S>>, S> DeviceQueueCreateInfoArray<'a, A, S> {
    pub(crate) fn new(infos: A::InitStorage) -> Self {
        Self {
            infos,
            _a: PhantomData,
            _scope: PhantomData,
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
pub struct DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage, S> {
    family_index: u32,
    to_write: &'initializer mut crate::array_storage::UninitArrayInitializer<
        'storage,
        DeviceQueueCreateInfo<'params, S>,
    >,
    pub family_properties: &'properties QueueFamilyProperties<S>,
}

impl<'params, 'properties, 'initializer, 'storage, S>
    DeviceQueueCreateInfoConfiguration<'params, 'properties, 'initializer, 'storage, S>
{
    /// internal only method to be called from [QueueFamilies::create_info_builder_iter]
    /// should pass the current queue_family_index, and set queue_count to max possible
    pub(crate) fn new(
        family_index: u32,
        to_write: &'initializer mut crate::array_storage::UninitArrayInitializer<
            'storage,
            DeviceQueueCreateInfo<'params, S>,
        >,
        family_properties: &'properties QueueFamilyProperties<S>,
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
        check_vuids::check_vuids!(DeviceQueueCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "queueFamilyIndex must be less than pQueueFamilyPropertyCount returned by vkGetPhysicalDeviceQueueFamilyProperties"
            }

            // Self.family_index should be valid from QueueFamilies.configure_create_info
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_00382: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "queueCount must be less than or equal to the queueCount member of the VkQueueFamilyProperties"
            "structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties in the pQueueFamilyProperties[queueFamilyIndex]"
            }

            assert!(priorities.len() <= self.family_properties.queue_count as usize)
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "Each element of pQueuePriorities must be between 0.0 and 1.0 inclusive"
            }

            // [QueuePriorities]
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_02861: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the protectedMemory feature is not enabled, the VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT"
            "bit of flags must not be set"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_06449: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If flags includes VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT, queueFamilyIndex must be the"
            "index of a queue family that includes the VK_QUEUE_PROTECTED_BIT capability"
            }

            assert!(
                !flags.contains(vk::DeviceQueueCreateFlags::PROTECTED_BIT),
                "must not push_config with PROTECTED_BIT. use push_config_with_protected instead"
            );
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "sType must be VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pNext must be NULL or a pointer to a valid instance of VkDeviceQueueGlobalPriorityCreateInfoKHR"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_unique: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "The sType value of each struct in the pNext chain must be unique"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "flags must be a valid combination of VkDeviceQueueCreateFlagBits values"
            }

            // vk::DeviceQueueCreateFlags, and checking VUID_VkDeviceQueueCreateInfo_flags_06449 above
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pQueuePriorities must be a valid pointer to an array of queueCount float values"
            }

            // QueuePriorities
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "queueCount must be greater than 0"
            }

            // QueuePriorities
        }

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
            _scope: PhantomData,
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
        check_vuids::check_vuids!(DeviceQueueCreateInfo);

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueFamilyIndex_00381: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "queueFamilyIndex must be less than pQueueFamilyPropertyCount returned by vkGetPhysicalDeviceQueueFamilyProperties"
            }

            // Self.family_index should be valid from QueueFamilies.configure_create_info
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_00382: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "queueCount must be less than or equal to the queueCount member of the VkQueueFamilyProperties"
            "structure, as returned by vkGetPhysicalDeviceQueueFamilyProperties in the pQueueFamilyProperties[queueFamilyIndex]"
            }

            assert!(
                priorities_for_non_protected.len() + priorities_for_protected.len()
                    <= self.family_properties.queue_count as usize
            );
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_00383: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "Each element of pQueuePriorities must be between 0.0 and 1.0 inclusive"
            }

            // [QueuePriorities]
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_02861: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the protectedMemory feature is not enabled, the VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT"
            "bit of flags must not be set"
            }

            // TODO: features not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_06449: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If flags includes VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT, queueFamilyIndex must be the"
            "index of a queue family that includes the VK_QUEUE_PROTECTED_BIT capability"
            }

            assert!(self
                .family_properties
                .queue_flags
                .contains(vk::QueueFlags::PROTECTED_BIT));
            assert!(
                !flags_for_non_protected.contains(vk::DeviceQueueCreateFlags::PROTECTED_BIT),
                "flags_for_non_protected should not include PROTECTED_BIT"
            );
            assert!(
                flags_for_protected.contains(vk::DeviceQueueCreateFlags::PROTECTED_BIT),
                "flags_for_protected must include PROTECTED_BIT"
            );
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "sType must be VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO"
            }

            // set below
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pNext must be NULL or a pointer to a valid instance of VkDeviceQueueGlobalPriorityCreateInfoKHR"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_sType_unique: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "The sType value of each struct in the pNext chain must be unique"
            }

            // TODO: p_next not supported
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_flags_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "flags must be a valid combination of VkDeviceQueueCreateFlagBits values"
            }

            // vk::DeviceQueueCreateFlags, and checking VUID_VkDeviceQueueCreateInfo_flags_06449 above
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_pQueuePriorities_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pQueuePriorities must be a valid pointer to an array of queueCount float values"
            }

            // QueuePriorities
        }

        #[allow(unused_labels)]
        'VUID_VkDeviceQueueCreateInfo_queueCount_arraylength: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "queueCount must be greater than 0"
            }

            // QueuePriorities
        }

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
                _scope: PhantomData,
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
                _scope: PhantomData,
            };
            self.to_write.push(protected_info)?;
        }

        Ok(())
    }
}
