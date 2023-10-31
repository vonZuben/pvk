use super::*;
use vk_safe_sys as vk;

use crate::physical_device::MemoryTypeChoice;
use crate::scope::{ScopeId, ScopeLife};

use vk::has_command::{AllocateMemory, FreeMemory};

use std::mem::MaybeUninit;
use std::ops::Deref;

pub trait DeviceMemoryConfig: Deref<Target = Self::Device> {
    type FreeProvider;
    type Commands: FreeMemory<Self::FreeProvider>;
    type Device: Device<Commands = Self::Commands>;
}

pub struct Config<D, F> {
    device: D,
    free_provider: PhantomData<F>,
}

impl<D: Device, F> DeviceMemoryConfig for Config<D, F>
where
    D::Commands: FreeMemory<F>,
{
    type FreeProvider = F;
    type Commands = D::Commands;
    type Device = D;
}

impl<D: Device, F> Deref for Config<D, F> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

pub trait DeviceMemory: Deref<Target = DeviceMemoryType<Self::Config>> {
    type Config: DeviceMemoryConfig<Device = Self::Device>;
    type Device;
}

pub struct DeviceMemoryType<D: DeviceMemoryConfig> {
    pub(crate) handle: vk::DeviceMemory,
    device: D,
}

impl<D: DeviceMemoryConfig> DeviceMemoryType<D> {
    fn new(handle: vk::DeviceMemory, device: D) -> Self {
        Self { handle, device }
    }
}

impl<D: DeviceMemoryConfig> std::fmt::Debug for DeviceMemoryType<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl<'d, 'pd, C: DeviceConfig, Pd: PhysicalDevice + ScopeLife<'pd>> ScopedDeviceType<'d, C, Pd> {
    pub fn allocate_memory<P, F>(
        &self,
        info: &MemoryAllocateInfo<'pd>,
    ) -> Result<DeviceMemoryType<Config<Self, F>>, vk::Result>
    where
        C::Commands: AllocateMemory<P> + FreeMemory<F>,
    {
        let fptr = self.commands.AllocateMemory().get_fptr();
        let mut memory = MaybeUninit::uninit();
        unsafe {
            let ret = fptr(
                self.handle,
                &info.inner,
                std::ptr::null(),
                memory.as_mut_ptr(),
            );
            check_raw_err!(ret);
            Ok(DeviceMemoryType::new(
                memory.assume_init(),
                Config {
                    device: *self,
                    free_provider: PhantomData,
                },
            ))
        }
    }
}

impl<D: DeviceMemoryConfig> Drop for DeviceMemoryType<D> {
    fn drop(&mut self) {
        let fptr = self.device.commands.FreeMemory().get_fptr();
        unsafe {
            fptr(self.device.handle, self.handle, std::ptr::null());
        }
    }
}

pub struct MemoryAllocateInfo<'pd> {
    inner: vk::MemoryAllocateInfo,
    _pd: ScopeId<'pd>,
}

impl<'pd> MemoryAllocateInfo<'pd> {
    pub const fn new(
        size: std::num::NonZeroU64,
        memory_type_choice: MemoryTypeChoice<'pd>,
    ) -> Self {
        check_vuid_defs2!( MemoryAllocateInfo
            pub const VUID_VkMemoryAllocateInfo_allocationSize_00638: &'static [u8] =
                "allocationSize must be greater than 0".as_bytes();
            pub const VUID_VkMemoryAllocateInfo_pNext_00639 : & 'static [ u8 ] = "If the pNext chain includes a VkExportMemoryAllocateInfo     structure, and any of the handle types specified in     VkExportMemoryAllocateInfo::handleTypes require a dedicated     allocation, as reported by     vkGetPhysicalDeviceImageFormatProperties2 in     VkExternalImageFormatProperties::externalMemoryProperties.externalMemoryFeatures     or     VkExternalBufferProperties::externalMemoryProperties.externalMemoryFeatures,     the pNext chain must include a ifdef::VK_KHR_dedicated_allocation[VkMemoryDedicatedAllocateInfo]" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_00640 : & 'static [ u8 ] = "If the pNext chain includes a VkExportMemoryAllocateInfo structure, it must not include a VkExportMemoryAllocateInfoNV or VkExportMemoryWin32HandleInfoNV structure" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_00641 : & 'static [ u8 ] = "If the pNext chain includes a VkImportMemoryWin32HandleInfoKHR structure, it must not include a VkImportMemoryWin32HandleInfoNV structure" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_allocationSize_01742 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle specified was created by the Vulkan API, and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT_KHR, then the values of allocationSize and memoryTypeIndex must match those specified when the memory object being imported was created" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_00648 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a POSIX file descriptor created outside of the Vulkan API, the value of memoryTypeIndex must be one of those returned by vkGetMemoryFdPropertiesKHR" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_None_00643 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle specified was created by the Vulkan API, the device mask specified by VkMemoryAllocateFlagsInfo must match that specified when the memory object being imported was allocated" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_None_00644 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle specified was created by the Vulkan API, the list of physical devices that comprise the logical device passed to vkAllocateMemory must match the list of physical devices that comprise the logical device on which the memory was originally allocated" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_00645 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is an NT handle or a global share handle created outside of the Vulkan API, the value of memoryTypeIndex must be one of those returned by vkGetMemoryWin32HandlePropertiesKHR" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_allocationSize_01743 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle was created by the Vulkan API, and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR or VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR, then the values of allocationSize and memoryTypeIndex must match those specified when the memory object being imported was created" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_allocationSize_00646 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_BIT, VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_KMT_BIT, or VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_RESOURCE_BIT, allocationSize must match the size reported in the memory requirements of the image or buffer member of the VkDedicatedAllocationMemoryAllocateInfoNV structure included in the pNext chain" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_allocationSize_00647 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT, allocationSize must match the size specified when creating the Direct3D 12 heap from which the external handle was extracted" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_01872 : & 'static [ u8 ] = "If the protected memory feature is not enabled, the VkMemoryAllocateInfo::memoryTypeIndex must not indicate a memory type that reports VK_MEMORY_PROPERTY_PROTECTED_BIT" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_01744 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a host pointer, the value of memoryTypeIndex must be one of those returned by vkGetMemoryHostPointerPropertiesEXT" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_allocationSize_01745 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a host pointer, allocationSize must be an integer multiple of VkPhysicalDeviceExternalMemoryHostPropertiesEXT::minImportedHostPointerAlignment" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02805 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a host pointer, the pNext chain must not include a VkDedicatedAllocationMemoryAllocateInfoNV structure with either its image or buffer field set to a value other than VK_NULL_HANDLE" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02806 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a host pointer, the pNext chain must not include a VkMemoryDedicatedAllocateInfo structure with either its image or buffer field set to a value other than VK_NULL_HANDLE" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_allocationSize_02383 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID, allocationSize must be the size returned by vkGetAndroidHardwareBufferPropertiesANDROID for the Android hardware buffer" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02384 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID, and the pNext chain does not include a VkMemoryDedicatedAllocateInfo structure or VkMemoryDedicatedAllocateInfo::image is VK_NULL_HANDLE, the Android hardware buffer must have a AHardwareBuffer_Desc::format of AHARDWAREBUFFER_FORMAT_BLOB and a AHardwareBuffer_Desc::usage that includes AHARDWAREBUFFER_USAGE_GPU_DATA_BUFFER" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_02385 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID, memoryTypeIndex must be one of those returned by vkGetAndroidHardwareBufferPropertiesANDROID for the Android hardware buffer" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_01874 : & 'static [ u8 ] = "If the parameters do not define an import operation, and the pNext chain includes a VkExportMemoryAllocateInfo structure with VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID included in its handleTypes member, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with image not equal to VK_NULL_HANDLE, then allocationSize must be 0, otherwise allocationSize must be greater than 0" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02386 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, the Android hardware buffer&#8217;s AHardwareBuffer::usage must include at least one of AHARDWAREBUFFER_USAGE_GPU_COLOR_OUTPUT or AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02387 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, the format of image must be VK_FORMAT_UNDEFINED or the format returned by vkGetAndroidHardwareBufferPropertiesANDROID in VkAndroidHardwareBufferFormatPropertiesANDROID::format for the Android hardware buffer" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02388 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with image that is not VK_NULL_HANDLE, the width, height, and array layer dimensions of image and the Android hardware buffer&#8217;s AHardwareBuffer_Desc must be identical" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02389 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with image that is not VK_NULL_HANDLE, and the Android hardware buffer&#8217;s AHardwareBuffer::usage includes AHARDWAREBUFFER_USAGE_GPU_MIPMAP_COMPLETE, the image must have a complete mipmap chain" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02586 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with image that is not VK_NULL_HANDLE, and the Android hardware buffer&#8217;s AHardwareBuffer::usage does not include AHARDWAREBUFFER_USAGE_GPU_MIPMAP_COMPLETE, the image must have exactly one mipmap level" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_02390 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes a VkMemoryDedicatedAllocateInfo structure with image that is not VK_NULL_HANDLE, each bit set in the usage of image must be listed in AHardwareBuffer Usage Equivalence, and if there is a corresponding AHARDWAREBUFFER_USAGE bit listed that bit must be included in the Android hardware buffer&#8217;s AHardwareBuffer_Desc::usage" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_opaqueCaptureAddress_03329 : & 'static [ u8 ] = "If VkMemoryOpaqueCaptureAddressAllocateInfo::opaqueCaptureAddress is not zero, VkMemoryAllocateFlagsInfo::flags must include VK_MEMORY_ALLOCATE_DEVICE_ADDRESS_CAPTURE_REPLAY_BIT" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_flags_03330 : & 'static [ u8 ] = "If VkMemoryAllocateFlagsInfo::flags includes VK_MEMORY_ALLOCATE_DEVICE_ADDRESS_CAPTURE_REPLAY_BIT, the bufferDeviceAddressCaptureReplay feature must be enabled" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_flags_03331 : & 'static [ u8 ] = "If VkMemoryAllocateFlagsInfo::flags includes VK_MEMORY_ALLOCATE_DEVICE_ADDRESS_BIT, the bufferDeviceAddress feature must be enabled" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_opaqueCaptureAddress_03333 : & 'static [ u8 ] = "If the parameters define an import operation, VkMemoryOpaqueCaptureAddressAllocateInfo::opaqueCaptureAddress must be zero" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_pNext_03332 : & 'static [ u8 ] = "If the pNext chain includes a VkImportMemoryHostPointerInfoEXT structure, VkMemoryOpaqueCaptureAddressAllocateInfo::opaqueCaptureAddress must be zero" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_sType_sType: &'static [u8] =
                "sType must be VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO".as_bytes();
            pub const VUID_VkMemoryAllocateInfo_pNext_pNext : & 'static [ u8 ] = "Each pNext member of any structure (including this one) in the pNext chain must be either NULL or a pointer to a valid instance of VkDedicatedAllocationMemoryAllocateInfoNV, VkExportMemoryAllocateInfo, VkExportMemoryAllocateInfoNV, VkExportMemoryWin32HandleInfoKHR, VkExportMemoryWin32HandleInfoNV, VkImportAndroidHardwareBufferInfoANDROID, VkImportMemoryFdInfoKHR, VkImportMemoryHostPointerInfoEXT, VkImportMemoryWin32HandleInfoKHR, VkImportMemoryWin32HandleInfoNV, VkMemoryAllocateFlagsInfo, VkMemoryDedicatedAllocateInfo, VkMemoryOpaqueCaptureAddressAllocateInfo, or VkMemoryPriorityAllocateInfoEXT" . as_bytes ( ) ;
            pub const VUID_VkMemoryAllocateInfo_sType_unique: &'static [u8] =
                "The sType value of each struct in the pNext chain must be unique".as_bytes();
        );

        let inner = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: size.get(),
            memory_type_index: memory_type_choice.index,
        };

        Self {
            inner,
            _pd: ScopeId::new(),
        }
    }
}
