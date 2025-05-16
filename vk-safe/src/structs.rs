/// Include a module, and publicly use the modules contents
macro_rules! pub_use_structs {
    (
        $( #[cfg($feature:ident)] $block:tt );* $(;)?
    ) => {
        mod inner {
            $( pub_use_structs!(@MOD $feature $block); )*
        }

        $( pub_use_structs!(@USE $block); )*
    };
    (
        @MOD
        $feature:ident
        {
            $(
                $(#[$($attributes:tt)*])*
                $name:ident
            );*
            $(;)?
        }
    ) => {
        $(
            #[cfg($feature)]
            #[allow(non_snake_case)]
            $(#[$($attributes)*])*
            pub mod $name;
        )*
    };

    (
        @USE
        {
            $(
                $(#[$($attributes:tt)*])*
                $name:ident
            );*
            $(;)?
        }
    ) => {
        $(
            #[allow(unused_imports)]
            pub use inner::$name::*;
        )*
    };
}

pub_use_structs!(
    #[cfg(VK_VERSION_1_0)]
    {
        ApplicationInfo;
        CommandBufferAllocInfo;
        CommandPoolCreateInfo;
        ExtensionProperties;
        FormatProperties;
        ImageFormatProperties;
        InstanceCreateInfo;
        LayerProperties;
        MappedMemoryRange;
        MemoryAllocateInfo;
        QueueFamilies;
        ShaderModuleCreateInfo;
        SparseImageFormatProperties;
        ImageParameters;
        DeviceCreateInfo;
        DeviceQueueCreateInfo;
        PhysicalDeviceFeatures;
        PhysicalDeviceMemoryProperties;
        PhysicalDeviceProperties;
    };

    #[cfg(VK_VERSION_1_1)]
    {
        PhysicalDeviceProperties2;
        PhysicalDeviceFeatures2;
        PhysicalDeviceVariablePointerFeatures;
    };

    #[cfg(VK_EXT_external_memory_host)]
    {
        PhysicalDeviceExternalMemoryHostPropertiesExt;
    };
);
