use super::*;
use vk_safe_sys as vk;
use vk::GetCommand;
use vk::VkEnumVariant;

use crate::scope::{ScopeLife, ScopeId};
use crate::physical_device::MemoryTypeChoice;

use std::mem::MaybeUninit;

pub struct DeviceMemory<'d, C: DeviceConfig, Pd: Scoped> where C::Commands: GetCommand<vk::FreeMemory> {
    pub(crate) handle: vk::DeviceMemory,
    pub(crate) device: ScopeDevice<'d, C, Pd>,
}

impl<'d, C: DeviceConfig, Pd: Scoped> DeviceMemory<'d, C, Pd> where C::Commands: GetCommand<vk::FreeMemory> {
    fn new(handle: vk::DeviceMemory, device: ScopeDevice<'d, C, Pd>) -> Self {
        Self { handle, device }
    }
}

impl<'d, C: DeviceConfig, Pd: Scoped> std::fmt::Debug for DeviceMemory<'_, C, Pd> where C::Commands: GetCommand<vk::FreeMemory> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl<'d, 'pd, C: DeviceConfig, Pd: ScopeLife<'pd>> ScopeDevice<'d, C, Pd> where C::Commands: GetCommand<vk::AllocateMemory> + GetCommand<vk::FreeMemory> {
    pub fn allocate_memory(&self, info: &MemoryAllocateInfo<'pd>) -> Result<DeviceMemory<'d, C, Pd>, vk::Result> {
        let fptr = get_fptr!(C vk::AllocateMemory, self);
        let mut memory = MaybeUninit::uninit();
        unsafe {
            let ret = fptr(self.handle, &info.inner, std::ptr::null(), memory.as_mut_ptr());
            check_raw_err!(ret);
            Ok(DeviceMemory::new(memory.assume_init(), *self))
        }
    }
}

impl<C: DeviceConfig, Pd: Scoped> Drop for DeviceMemory<'_, C, Pd> where C::Commands: GetCommand<vk::FreeMemory> {
    fn drop(&mut self) {
        let fptr = get_fptr!(C vk::FreeMemory, self.device);
        unsafe { fptr((*self.device).handle, self.handle, std::ptr::null()); }
    }
}

pub struct MemoryAllocateInfo<'pd> {
    inner: vk::MemoryAllocateInfo,
    _pd: ScopeId<'pd>,
}

impl<'pd> MemoryAllocateInfo<'pd> {
    pub fn new(size: std::num::NonZeroU64, memory_type_choice: MemoryTypeChoice<'_, 'pd>) -> Self {
        alloc_info_verify::Vuid::verify();

        {
            use vk::memory_property_flag_bits::PROTECTED_BIT;
            use vk::BitList;
            assert!(!memory_type_choice.ty.property_flags.contains(bitmask!(PROTECTED_BIT).bitmask()),
                "Protected memory is not currently supported");
        }

        let inner = vk::MemoryAllocateInfo {
            s_type: vk::structure_type::MEMORY_ALLOCATE_INFO.as_enum(),
            p_next: std::ptr::null(),
            allocation_size: size.get(),
            memory_type_index: memory_type_choice.index,
        };

        Self { inner, _pd: Default::default() }
    }
}

mod alloc_info_verify {
    use vk_safe_sys::validation::MemoryAllocateInfo::*;

    verify_vuids!(
        pub Vuid()
        {
            const VUID_VkMemoryAllocateInfo_allocationSize_00638: () = {
                // using std::num::NonZeroU64
            };

            const VUID_VkMemoryAllocateInfo_pNext_00639: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
            };

            const VUID_VkMemoryAllocateInfo_pNext_00640: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
            };

            const VUID_VkMemoryAllocateInfo_pNext_00641: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
            };

            const VUID_VkMemoryAllocateInfo_allocationSize_01742: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_memoryTypeIndex_00648: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_None_00643: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_None_00644: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_memoryTypeIndex_00645: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_allocationSize_01743: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_allocationSize_00646: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_allocationSize_00647: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_memoryTypeIndex_01872: () = {
                // there is a hard check that denies use of protected memory for now
            };

            const VUID_VkMemoryAllocateInfo_memoryTypeIndex_01744: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_allocationSize_01745: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_allocationSize_02383: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_02384: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_memoryTypeIndex_02385: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_01874: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_02386: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_02387: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_02388: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_02389: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_02586: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_pNext_02390: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // related to "import operation"
            };

            const VUID_VkMemoryAllocateInfo_sType_sType: () = {
                // set in new()
            };

            const VUID_VkMemoryAllocateInfo_pNext_pNext: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // VALID PNEXT MEMBERS
            };

            const VUID_VkMemoryAllocateInfo_sType_unique: () = {
                //=============================================
                //================Add when p_next is supported=
                //=============================================
                // UNIQUE
            };
        }
    );

    check_vuid_defs!(
        pub const VUID_VkMemoryAllocateInfo_allocationSize_00638: &'static [u8] =
            "allocationSize must be greater than 0".as_bytes();
        pub const VUID_VkMemoryAllocateInfo_pNext_00639 : & 'static [ u8 ] = "If the pNext chain contains an instance of     VkExportMemoryAllocateInfo, and any of the handle types specified     in VkExportMemoryAllocateInfo::handleTypes require a     dedicated allocation, as reported by     vkGetPhysicalDeviceImageFormatProperties2 in     VkExternalImageFormatProperties::externalMemoryProperties::externalMemoryFeatures     or     VkExternalBufferProperties::externalMemoryProperties::externalMemoryFeatures,     the pNext chain must contain an instance of ifdef::VK_KHR_dedicated_allocation[VkMemoryDedicatedAllocateInfo]" . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_00640 : & 'static [ u8 ] = "If the pNext chain contains an instance of VkExportMemoryAllocateInfo, it must not contain an instance of VkExportMemoryAllocateInfoNV or VkExportMemoryWin32HandleInfoNV." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_00641 : & 'static [ u8 ] = "If the pNext chain contains an instance of VkImportMemoryWin32HandleInfoKHR, it must not contain an instance of VkImportMemoryWin32HandleInfoNV." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_allocationSize_01742 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle specified was created by the Vulkan API, and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT_KHR, then the values of allocationSize and memoryTypeIndex must match those specified when the memory object being imported was created." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_00648 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a POSIX file descriptor created outside of the Vulkan API, the value of memoryTypeIndex must be one of those returned by vkGetMemoryFdPropertiesKHR." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_None_00643 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle specified was created by the Vulkan API, the device mask specified by VkMemoryAllocateFlagsInfo must match that specified when the memory object being imported was allocated." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_None_00644 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle specified was created by the Vulkan API, the list of physical devices that comprise the logical device passed to vkAllocateMemory must match the list of physical devices that comprise the logical device on which the memory was originally allocated." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_00645 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is an NT handle or a global share handle created outside of the Vulkan API, the value of memoryTypeIndex must be one of those returned by vkGetMemoryWin32HandlePropertiesKHR." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_allocationSize_01743 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle was created by the Vulkan API, and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR or VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR, then the values of allocationSize and memoryTypeIndex must match those specified when the memory object being imported was created." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_allocationSize_00646 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_BIT, VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_KMT_BIT, or VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_RESOURCE_BIT, allocationSize must match the size reported in the memory requirements of the image or buffer member of the instance of VkDedicatedAllocationMemoryAllocateInfoNV included in the pNext chain." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_allocationSize_00647 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT, allocationSize must match the size specified when creating the Direct3D 12 heap from which the external handle was extracted." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_01872 : & 'static [ u8 ] = "If the protected memory feature is not enabled, the VkMemoryAllocateInfo::memoryTypeIndex must not indicate a memory type that reports VK_MEMORY_PROPERTY_PROTECTED_BIT." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_01744 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a host pointer, the value of memoryTypeIndex must be one of those returned by vkGetMemoryHostPointerPropertiesEXT" . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_allocationSize_01745 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle is a host pointer, allocationSize must be an integer multiple of VkPhysicalDeviceExternalMemoryHostPropertiesEXT::minImportedHostPointerAlignment" . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_allocationSize_02383 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID, allocationSize must be the size returned by vkGetAndroidHardwareBufferPropertiesANDROID for the Android hardware buffer." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_02384 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID, and the pNext chain does not contain an instance of VkMemoryDedicatedAllocateInfo or VkMemoryDedicatedAllocateInfo::image is VK_NULL_HANDLE, the Android hardware buffer must have a AHardwareBuffer_Desc::format of AHARDWAREBUFFER_FORMAT_BLOB and a AHardwareBuffer_Desc::usage that includes AHARDWAREBUFFER_USAGE_GPU_DATA_BUFFER." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_memoryTypeIndex_02385 : & 'static [ u8 ] = "If the parameters define an import operation and the external handle type is VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID, memoryTypeIndex must be one of those returned by vkGetAndroidHardwareBufferPropertiesANDROID for the Android hardware buffer." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_01874 : & 'static [ u8 ] = "If the parameters do not define an import operation, and the pNext chain contains an instance of VkExportMemoryAllocateInfo with VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID included in its handleTypes member, and the pNext contains an instance of VkMemoryDedicatedAllocateInfo with image not equal to VK_NULL_HANDLE, then allocationSize must be 0, otherwise allocationSize must be greater than 0." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_02386 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes an instance of VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, the Android hardware buffer&#8217;s AHardwareBuffer::usage must include at least one of AHARDWAREBUFFER_USAGE_GPU_COLOR_OUTPUT or AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_02387 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes an instance of VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, the format of image must be VK_FORMAT_UNDEFINED or the format returned by vkGetAndroidHardwareBufferPropertiesANDROID in VkAndroidHardwareBufferFormatPropertiesANDROID::format for the Android hardware buffer." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_02388 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes an instance of VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, the width, height, and array layer dimensions of image and the Android hardware buffer&#8217;s AHardwareBuffer_Desc must be identical." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_02389 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes an instance of VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, and the Android hardware buffer&#8217;s AHardwareBuffer::usage includes AHARDWAREBUFFER_USAGE_GPU_MIPMAP_COMPLETE, the image must have a complete mipmap chain." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_02586 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes an instance of VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, and the Android hardware buffer&#8217;s AHardwareBuffer::usage does not include AHARDWAREBUFFER_USAGE_GPU_MIPMAP_COMPLETE, the image must have exactly one mipmap level." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_pNext_02390 : & 'static [ u8 ] = "If the parameters define an import operation, the external handle is an Android hardware buffer, and the pNext chain includes an instance of VkMemoryDedicatedAllocateInfo with image that is not VK_NULL_HANDLE, each bit set in the usage of image must be listed in AHardwareBuffer Usage Equivalence, and if there is a corresponding AHARDWAREBUFFER_USAGE bit listed that bit must be included in the Android hardware buffer&#8217;s AHardwareBuffer_Desc::usage." . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_sType_sType: &'static [u8] =
            "sType must be VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO".as_bytes();
        pub const VUID_VkMemoryAllocateInfo_pNext_pNext : & 'static [ u8 ] = "Each pNext member of any structure (including this one) in the pNext chain must be either NULL or a pointer to a valid instance of VkDedicatedAllocationMemoryAllocateInfoNV, VkExportMemoryAllocateInfo, VkExportMemoryAllocateInfoNV, VkExportMemoryWin32HandleInfoKHR, VkExportMemoryWin32HandleInfoNV, VkImportAndroidHardwareBufferInfoANDROID, VkImportMemoryFdInfoKHR, VkImportMemoryHostPointerInfoEXT, VkImportMemoryWin32HandleInfoKHR, VkImportMemoryWin32HandleInfoNV, VkMemoryAllocateFlagsInfo, VkMemoryDedicatedAllocateInfo, or VkMemoryPriorityAllocateInfoEXT" . as_bytes ( ) ;
        pub const VUID_VkMemoryAllocateInfo_sType_unique: &'static [u8] =
            "Each sType member in the pNext chain must be unique".as_bytes();
    );
}