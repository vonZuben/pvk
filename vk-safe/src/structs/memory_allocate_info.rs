use std::marker::PhantomData;
use std::ops::Deref;

use crate::type_conversions::ConvertWrapper;

use super::physical_device_memory_properties::MemoryTypeChoice;

use vk_safe_sys as vk;
pub struct MemoryAllocateInfo<S, P, H> {
    inner: vk::MemoryAllocateInfo,
    pd: PhantomData<S>,
    property_flags: PhantomData<P>,
    heap_flags: PhantomData<H>,
}

unsafe impl<S, P, H> ConvertWrapper<vk::MemoryAllocateInfo> for MemoryAllocateInfo<S, P, H> {}

impl<S, P, H> Deref for MemoryAllocateInfo<S, P, H> {
    type Target = vk::MemoryAllocateInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S, P, H> MemoryAllocateInfo<S, P, H> {
    pub const fn new(
        size: std::num::NonZeroU64,
        memory_type_choice: MemoryTypeChoice<S, P, H>,
    ) -> Self {
        #![allow(unused_labels)]
        check_vuids::check_vuids!(MemoryAllocateInfo);

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_07897: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters do not define an import or export operation, allocationSize must"
            "be greater than 0"
            }

            // using NonZeroU64
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_None_06657: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "The parameters must not define more than one import operation"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_07899: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an export operation and the handle type is not VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID"
            ", allocationSize must be greater than 0"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_buffer_06380: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation from an VkBufferCollectionFUCHSIA, and"
            "VkMemoryDedicatedAllocateInfo::buffer is present and non-NULL, VkImportMemoryBufferCollectionFUCHSIA::collection"
            "and VkImportMemoryBufferCollectionFUCHSIA::index must match VkBufferCollectionBufferCreateInfoFUCHSIA::collection"
            "and VkBufferCollectionBufferCreateInfoFUCHSIA::index, respectively, of the VkBufferCollectionBufferCreateInfoFUCHSIA"
            "structure used to create the VkMemoryDedicatedAllocateInfo::buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_image_06381: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation from an VkBufferCollectionFUCHSIA, and"
            "VkMemoryDedicatedAllocateInfo::image is present and non-NULL, VkImportMemoryBufferCollectionFUCHSIA::collection"
            "and VkImportMemoryBufferCollectionFUCHSIA::index must match VkBufferCollectionImageCreateInfoFUCHSIA::collection"
            "and VkBufferCollectionImageCreateInfoFUCHSIA::index, respectively, of the VkBufferCollectionImageCreateInfoFUCHSIA"
            "structure used to create the VkMemoryDedicatedAllocateInfo::image"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_06382: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation from an VkBufferCollectionFUCHSIA, allocationSize"
            "must match VkMemoryRequirements::size value retrieved by vkGetImageMemoryRequirements"
            "or vkGetBufferMemoryRequirements for image-based or buffer-based collections respectively"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_06383: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation from an VkBufferCollectionFUCHSIA, the"
            "pNext chain must include a VkMemoryDedicatedAllocateInfo structure with either its"
            "image or buffer field set to a value other than VK_NULL_HANDLE"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_image_06384: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation from an VkBufferCollectionFUCHSIA and"
            "VkMemoryDedicatedAllocateInfo::image is not VK_NULL_HANDLE, the image must be created"
            "with a VkBufferCollectionImageCreateInfoFUCHSIA structure chained to its VkImageCreateInfo::pNext"
            "pointer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_buffer_06385: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation from an VkBufferCollectionFUCHSIA and"
            "VkMemoryDedicatedAllocateInfo::buffer is not VK_NULL_HANDLE, the buffer must be created"
            "with a VkBufferCollectionBufferCreateInfoFUCHSIA structure chained to its VkBufferCreateInfo::pNext"
            "pointer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_memoryTypeIndex_06386: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation from an VkBufferCollectionFUCHSIA, memoryTypeIndex"
            "must be from VkBufferCollectionPropertiesFUCHSIA as retrieved by vkGetBufferCollectionPropertiesFUCHSIA"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_00639: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkExportMemoryAllocateInfo structure, and any of the"
            "handle types specified in VkExportMemoryAllocateInfo::handleTypes require a dedicated"
            "allocation, as reported by vkGetPhysicalDeviceImageFormatProperties2 in VkExternalImageFormatProperties::externalMemoryProperties.externalMemoryFeatures,"
            "or by vkGetPhysicalDeviceExternalBufferProperties in VkExternalBufferProperties::externalMemoryProperties.externalMemoryFeatures,"
            "the pNext chain must include a VkMemoryDedicatedAllocateInfo or VkDedicatedAllocationMemoryAllocateInfoNV"
            "structure with either its image or buffer member set to a value other than VK_NULL_HANDLE"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_00640: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkExportMemoryAllocateInfo structure, it must not include"
            "a VkExportMemoryAllocateInfoNV or VkExportMemoryWin32HandleInfoNV structure"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_00641: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkImportMemoryWin32HandleInfoKHR structure, it must"
            "not include a VkImportMemoryWin32HandleInfoNV structure"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_01742: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle specified was created"
            "by the Vulkan API, and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT,"
            "then the values of allocationSize and memoryTypeIndex must match those specified when"
            "the payload being imported was created"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_None_00643: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle specified was"
            "created by the Vulkan API, the device mask specified by VkMemoryAllocateFlagsInfo"
            "must match the mask specified when the payload being imported was allocated"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_None_00644: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle specified was"
            "created by the Vulkan API, the list of physical devices that comprise the logical"
            "device passed to vkAllocateMemory must match the list of physical devices that comprise"
            "the logical device on which the payload was originally allocated"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_memoryTypeIndex_00645: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle is an NT handle"
            "or a global share handle created outside of the Vulkan API, the value of memoryTypeIndex"
            "must be one of those returned by vkGetMemoryWin32HandlePropertiesKHR"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_01743: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle was created by the"
            "Vulkan API, and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT"
            "or VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT, then the values of allocationSize"
            "and memoryTypeIndex must match those specified when the payload being imported was"
            "created"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_00647: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT,"
            "allocationSize must match the size specified when creating the Direct3D 12 heap from"
            "which the payload was extracted"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_memoryTypeIndex_00648: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle is a POSIX file"
            "descriptor created outside of the Vulkan API, the value of memoryTypeIndex must be"
            "one of those returned by vkGetMemoryFdPropertiesKHR"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_memoryTypeIndex_01872: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the protectedMemory feature is not enabled, the VkMemoryAllocateInfo::memoryTypeIndex"
            "must not indicate a memory type that reports VK_MEMORY_PROPERTY_PROTECTED_BIT"
            }

            // TODO: protectedMemory not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_memoryTypeIndex_01744: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle is a host pointer,"
            "the value of memoryTypeIndex must be one of those returned by vkGetMemoryHostPointerPropertiesEXT"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_01745: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle is a host pointer,"
            "allocationSize must be an integer multiple of VkPhysicalDeviceExternalMemoryHostPropertiesEXT::minImportedHostPointerAlignment"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02805: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle is a host pointer,"
            "the pNext chain must not include a VkDedicatedAllocationMemoryAllocateInfoNV structure"
            "with either its image or buffer field set to a value other than VK_NULL_HANDLE"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02806: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle is a host pointer,"
            "the pNext chain must not include a VkMemoryDedicatedAllocateInfo structure with either"
            "its image or buffer field set to a value other than VK_NULL_HANDLE"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_02383: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID,"
            "allocationSize must be the size returned by vkGetAndroidHardwareBufferPropertiesANDROID"
            "for the Android hardware buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02384: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID,"
            "and the pNext chain does not include a VkMemoryDedicatedAllocateInfo structure or"
            "VkMemoryDedicatedAllocateInfo::image is VK_NULL_HANDLE, the Android hardware buffer"
            "must have a AHardwareBuffer_Desc::format of AHARDWAREBUFFER_FORMAT_BLOB and a AHardwareBuffer_Desc::usage"
            "that includes AHARDWAREBUFFER_USAGE_GPU_DATA_BUFFER"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_memoryTypeIndex_02385: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID,"
            "memoryTypeIndex must be one of those returned by vkGetAndroidHardwareBufferPropertiesANDROID"
            "for the Android hardware buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_01874: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters do not define an import operation, and the pNext chain includes"
            "a VkExportMemoryAllocateInfo structure with VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID"
            "included in its handleTypes member, and the pNext chain includes a VkMemoryDedicatedAllocateInfo"
            "structure with image not equal to VK_NULL_HANDLE, then allocationSize must be 0"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_07900: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an export operation, the handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID,"
            "and the pNext does not include a VkMemoryDedicatedAllocateInfo structure, allocationSize"
            "must be greater than 0"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_07901: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an export operation, the handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID,"
            "and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with buffer"
            "set to a valid VkBuffer object, allocationSize must be greater than 0"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02386: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is an Android hardware"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo with image that"
            "is not VK_NULL_HANDLE, the Android hardware buffer&#8217;s AHardwareBuffer::usage"
            "must include at least one of AHARDWAREBUFFER_USAGE_GPU_FRAMEBUFFER, AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE"
            "or AHARDWAREBUFFER_USAGE_GPU_DATA_BUFFER"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02387: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is an Android hardware"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo with image that"
            "is not VK_NULL_HANDLE, the format of image must be VK_FORMAT_UNDEFINED or the format"
            "returned by vkGetAndroidHardwareBufferPropertiesANDROID in VkAndroidHardwareBufferFormatPropertiesANDROID::format"
            "for the Android hardware buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02388: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is an Android hardware"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with"
            "image that is not VK_NULL_HANDLE, the width, height, and array layer dimensions of"
            "image and the Android hardware buffer&#8217;s AHardwareBuffer_Desc must be identical"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02389: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is an Android hardware"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with"
            "image that is not VK_NULL_HANDLE, and the Android hardware buffer&#8217;s AHardwareBuffer::usage"
            "includes AHARDWAREBUFFER_USAGE_GPU_MIPMAP_COMPLETE, the image must have a complete"
            "mipmap chain"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02586: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is an Android hardware"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with"
            "image that is not VK_NULL_HANDLE, and the Android hardware buffer&#8217;s AHardwareBuffer::usage"
            "does not include AHARDWAREBUFFER_USAGE_GPU_MIPMAP_COMPLETE, the image must have exactly"
            "one mipmap level"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_02390: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is an Android hardware"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with"
            "image that is not VK_NULL_HANDLE, each bit set in the usage of image must be listed"
            "in AHardwareBuffer Usage Equivalence, and if there is a corresponding AHARDWAREBUFFER_USAGE"
            "bit listed that bit must be included in the Android hardware buffer&#8217;s AHardwareBuffer_Desc::usage"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_screenBufferImport_08941: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_SCREEN_BUFFER_BIT_QNX,"
            "VkPhysicalDeviceExternalMemoryScreenBufferFeaturesQNX::screenBufferImport must be"
            "enabled"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_08942: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_SCREEN_BUFFER_BIT_QNX,"
            "allocationSize must be the size returned by vkGetScreenBufferPropertiesQNX for the"
            "QNX Screen buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_memoryTypeIndex_08943: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_SCREEN_BUFFER_BIT_QNX,"
            "memoryTypeIndex must be one of those returned by vkGetScreenBufferPropertiesQNX for"
            "the QNX Screen buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_08944: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is a QNX Screen"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo with image that"
            "is not VK_NULL_HANDLE, the QNX Screen&#8217;s buffer must be a valid QNX Screen buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_08945: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is an QNX Screen"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo with image that"
            "is not VK_NULL_HANDLE, the format of image must be VK_FORMAT_UNDEFINED or the format"
            "returned by vkGetScreenBufferPropertiesQNX in VkScreenBufferFormatPropertiesQNX::format"
            "for the QNX Screen buffer"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_08946: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, the external handle is a QNX Screen"
            "buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with"
            "image that is not VK_NULL_HANDLE, the width, height, and array layer dimensions of"
            "image and the QNX Screen buffer&#8217;s _screen_buffer must be identical"
            }

            // TODO: import and export operation not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_opaqueCaptureAddress_03329: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkMemoryOpaqueCaptureAddressAllocateInfo::opaqueCaptureAddress is not zero, VkMemoryAllocateFlagsInfo::flags"
            "must include VK_MEMORY_ALLOCATE_DEVICE_ADDRESS_CAPTURE_REPLAY_BIT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_flags_03330: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkMemoryAllocateFlagsInfo::flags includes VK_MEMORY_ALLOCATE_DEVICE_ADDRESS_CAPTURE_REPLAY_BIT,"
            "the bufferDeviceAddressCaptureReplay feature must be enabled"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_flags_03331: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkMemoryAllocateFlagsInfo::flags includes VK_MEMORY_ALLOCATE_DEVICE_ADDRESS_BIT,"
            "the bufferDeviceAddress feature must be enabled"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_03332: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkImportMemoryHostPointerInfoEXT structure, VkMemoryOpaqueCaptureAddressAllocateInfo::opaqueCaptureAddress"
            "must be zero"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_opaqueCaptureAddress_03333: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation, VkMemoryOpaqueCaptureAddressAllocateInfo::opaqueCaptureAddress"
            "must be zero"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_None_04749: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ZIRCON_VMO_BIT_FUCHSIA,"
            "the value of memoryTypeIndex must be an index identifying a memory type from the memoryTypeBits"
            "field of the VkMemoryZirconHandlePropertiesFUCHSIA structure populated by a call to"
            "vkGetMemoryZirconHandlePropertiesFUCHSIA"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_07902: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ZIRCON_VMO_BIT_FUCHSIA,"
            "the value of allocationSize must be greater than 0"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_allocationSize_07903: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ZIRCON_VMO_BIT_FUCHSIA,"
            "the value of allocationSize must be less than or equal to the size of the VMO as determined"
            "by zx_vmo_get_size(handle) where handle is the VMO handle to the imported external"
            "memory"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_06780: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If the pNext chain includes a VkExportMetalObjectCreateInfoEXT structure, its exportObjectType"
            "member must be VK_EXPORT_METAL_OBJECT_TYPE_METAL_BUFFER_BIT_EXT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_sType_sType: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "sType must be VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO"
            }

            // set automatically below
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_pNext_pNext: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "Each pNext member of any structure (including this one) in the pNext chain must be"
            "either NULL or a pointer to a valid instance of VkDedicatedAllocationMemoryAllocateInfoNV,"
            "VkExportMemoryAllocateInfo, VkExportMemoryAllocateInfoNV, VkExportMemoryWin32HandleInfoKHR,"
            "VkExportMemoryWin32HandleInfoNV, VkExportMetalObjectCreateInfoEXT, VkImportAndroidHardwareBufferInfoANDROID,"
            "VkImportMemoryBufferCollectionFUCHSIA, VkImportMemoryFdInfoKHR, VkImportMemoryHostPointerInfoEXT,"
            "VkImportMemoryWin32HandleInfoKHR, VkImportMemoryWin32HandleInfoNV, VkImportMemoryZirconHandleInfoFUCHSIA,"
            "VkImportMetalBufferInfoEXT, VkImportScreenBufferInfoQNX, VkMemoryAllocateFlagsInfo,"
            "VkMemoryDedicatedAllocateInfo, VkMemoryOpaqueCaptureAddressAllocateInfo, or VkMemoryPriorityAllocateInfoEXT"
            }

            // TODO: p_next not currently supported
        }

        #[allow(unused_labels)]
        'VUID_VkMemoryAllocateInfo_sType_unique: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "The sType value of each struct in the pNext chain must be unique, with the exception"
            "of structures of type VkExportMetalObjectCreateInfoEXT"
            }

            // TODO: p_next not currently supported
        }

        let inner = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: size.get(),
            memory_type_index: memory_type_choice.index,
        };

        Self {
            inner,
            pd: PhantomData,
            property_flags: PhantomData,
            heap_flags: PhantomData,
        }
    }
}
