use crate::vk::PhysicalDeviceProperties;

use crate::type_conversions::ConvertWrapper;

struct_wrapper!(
/// VkPhysicalDeviceProperties2 - Structure specifying physical device properties
///
/// <https://registry.khronos.org/VulkanSC/specs/1.0-extensions/man/html/VkPhysicalDeviceProperties2.html>
PhysicalDeviceProperties2<Pd,>
);

struct_wrapper!(PhysicalDeviceExternalFormatResolvePropertiesANDROID<Pd,> impl Debug);

const _: () = {
    check_vuids::check_vuids!(PhysicalDeviceProperties2);

    #[allow(unused_labels)]
    'VUID_VkPhysicalDeviceProperties2_sType_sType: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "sType must be VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2"
        }

        // see struct_extensions.rs
    }

    #[allow(unused_labels)]
    'VUID_VkPhysicalDeviceProperties2_pNext_pNext: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "Each pNext member of any structure (including this one) in the pNext chain must be"
        "either NULL or a pointer to a valid instance of VkPhysicalDeviceAccelerationStructurePropertiesKHR,"
        "VkPhysicalDeviceBlendOperationAdvancedPropertiesEXT, VkPhysicalDeviceClusterCullingShaderPropertiesHUAWEI,"
        "VkPhysicalDeviceConservativeRasterizationPropertiesEXT, VkPhysicalDeviceCooperativeMatrixPropertiesKHR,"
        "VkPhysicalDeviceCooperativeMatrixPropertiesNV, VkPhysicalDeviceCopyMemoryIndirectPropertiesNV,"
        "VkPhysicalDeviceCustomBorderColorPropertiesEXT, VkPhysicalDeviceDepthStencilResolveProperties,"
        "VkPhysicalDeviceDescriptorBufferDensityMapPropertiesEXT, VkPhysicalDeviceDescriptorBufferPropertiesEXT,"
        "VkPhysicalDeviceDescriptorIndexingProperties, VkPhysicalDeviceDeviceGeneratedCommandsPropertiesNV,"
        "VkPhysicalDeviceDiscardRectanglePropertiesEXT, VkPhysicalDeviceDisplacementMicromapPropertiesNV,"
        "VkPhysicalDeviceDriverProperties, VkPhysicalDeviceDrmPropertiesEXT, VkPhysicalDeviceExtendedDynamicState3PropertiesEXT,"
        "VkPhysicalDeviceExtendedSparseAddressSpacePropertiesNV, VkPhysicalDeviceExternalFormatResolvePropertiesANDROID,"
        "VkPhysicalDeviceExternalMemoryHostPropertiesEXT, VkPhysicalDeviceFloatControlsProperties,"
        "VkPhysicalDeviceFragmentDensityMap2PropertiesEXT, VkPhysicalDeviceFragmentDensityMapOffsetPropertiesQCOM,"
        "VkPhysicalDeviceFragmentDensityMapPropertiesEXT, VkPhysicalDeviceFragmentShaderBarycentricPropertiesKHR,"
        "VkPhysicalDeviceFragmentShadingRateEnumsPropertiesNV, VkPhysicalDeviceFragmentShadingRatePropertiesKHR,"
        "VkPhysicalDeviceGraphicsPipelineLibraryPropertiesEXT, VkPhysicalDeviceHostImageCopyPropertiesEXT,"
        "VkPhysicalDeviceIDProperties, VkPhysicalDeviceImageProcessing2PropertiesQCOM, VkPhysicalDeviceImageProcessingPropertiesQCOM,"
        "VkPhysicalDeviceInlineUniformBlockProperties, VkPhysicalDeviceLayeredDriverPropertiesMSFT,"
        "VkPhysicalDeviceLineRasterizationPropertiesEXT, VkPhysicalDeviceMaintenance3Properties,"
        "VkPhysicalDeviceMaintenance4Properties, VkPhysicalDeviceMaintenance5PropertiesKHR,"
        "VkPhysicalDeviceMemoryDecompressionPropertiesNV, VkPhysicalDeviceMeshShaderPropertiesEXT,"
        "VkPhysicalDeviceMeshShaderPropertiesNV, VkPhysicalDeviceMultiDrawPropertiesEXT, VkPhysicalDeviceMultiviewPerViewAttributesPropertiesNVX,"
        "VkPhysicalDeviceMultiviewProperties, VkPhysicalDeviceNestedCommandBufferPropertiesEXT,"
        "VkPhysicalDeviceOpacityMicromapPropertiesEXT, VkPhysicalDeviceOpticalFlowPropertiesNV,"
        "VkPhysicalDevicePCIBusInfoPropertiesEXT, VkPhysicalDevicePerformanceQueryPropertiesKHR,"
        "VkPhysicalDevicePipelineRobustnessPropertiesEXT, VkPhysicalDevicePointClippingProperties,"
        "VkPhysicalDevicePortabilitySubsetPropertiesKHR, VkPhysicalDeviceProtectedMemoryProperties,"
        "VkPhysicalDeviceProvokingVertexPropertiesEXT, VkPhysicalDevicePushDescriptorPropertiesKHR,"
        "VkPhysicalDeviceRayTracingInvocationReorderPropertiesNV, VkPhysicalDeviceRayTracingPipelinePropertiesKHR,"
        "VkPhysicalDeviceRayTracingPropertiesNV, VkPhysicalDeviceRobustness2PropertiesEXT,"
        "VkPhysicalDeviceSampleLocationsPropertiesEXT, VkPhysicalDeviceSamplerFilterMinmaxProperties,"
        "VkPhysicalDeviceShaderCoreBuiltinsPropertiesARM, VkPhysicalDeviceShaderCoreProperties2AMD,"
        "VkPhysicalDeviceShaderCorePropertiesAMD, VkPhysicalDeviceShaderCorePropertiesARM,"
        "VkPhysicalDeviceShaderEnqueuePropertiesAMDX, VkPhysicalDeviceShaderIntegerDotProductProperties,"
        "VkPhysicalDeviceShaderModuleIdentifierPropertiesEXT, VkPhysicalDeviceShaderObjectPropertiesEXT,"
        "VkPhysicalDeviceShaderSMBuiltinsPropertiesNV, VkPhysicalDeviceShaderTileImagePropertiesEXT,"
        "VkPhysicalDeviceShadingRateImagePropertiesNV, VkPhysicalDeviceSubgroupProperties,"
        "VkPhysicalDeviceSubgroupSizeControlProperties, VkPhysicalDeviceSubpassShadingPropertiesHUAWEI,"
        "VkPhysicalDeviceTexelBufferAlignmentProperties, VkPhysicalDeviceTimelineSemaphoreProperties,"
        "VkPhysicalDeviceTransformFeedbackPropertiesEXT, VkPhysicalDeviceVertexAttributeDivisorPropertiesEXT,"
        "VkPhysicalDeviceVulkan11Properties, VkPhysicalDeviceVulkan12Properties, or VkPhysicalDeviceVulkan13Properties"
        }

        // see struct_extensions.rs
    }

    #[allow(unused_labels)]
    'VUID_VkPhysicalDeviceProperties2_sType_unique: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "The sType value of each struct in the pNext chain must be unique"
        }

        // compile_error!("new VUID");
    }
};

impl<Pd> std::ops::Deref for PhysicalDeviceProperties2<Pd> {
    type Target = PhysicalDeviceProperties<Pd>;

    fn deref(&self) -> &Self::Target {
        unsafe { ConvertWrapper::from_c(&self.inner.properties) }
    }
}

impl<Pd> std::fmt::Debug for PhysicalDeviceProperties2<Pd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}
