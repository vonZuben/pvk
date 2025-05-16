/// Include a module, and publicly use the modules contents
macro_rules! pub_use_structs {
    (
        $( #[cfg($feature:ident)] $block:tt );* $(;)?
    ) => {
        mod inner {
            $( pub_use_structs!(@MOD $feature $block); )*
        }

        $( pub_use_structs!(@USE $feature $block); )*
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
            #[allow(unused_imports)]
            pub use inner::$name::*;
        )*
    };
}

pub_use_structs!(
    #[cfg(VK_VERSION_1_0)]
    {
        ApplicationInfo;
        CommandBufferAllocateInfo;
        CommandPoolCreateInfo;
        ExtensionProperties;
        FormatProperties;
        ImageFormatProperties;
        InstanceCreateInfo;
        LayerProperties;
        MappedMemoryRange;
        MemoryAllocateInfo;
        MemoryHeap;
        MemoryType;
        ShaderModuleCreateInfo;
        SparseImageFormatProperties;
        ImageParameters;
        DeviceCreateInfo;
        DeviceQueueCreateInfo;
        PhysicalDeviceFeatures;
        PhysicalDeviceMemoryProperties;
        PhysicalDeviceProperties;

        // bespoke structs
        QueueFamilies;
    };

    #[cfg(VK_VERSION_1_1)]
    {
        PhysicalDeviceProperties2;
        PhysicalDeviceFeatures2;
        PhysicalDeviceVariablePointersFeatures;
    };

    #[cfg(VK_EXT_external_memory_host)]
    {
        PhysicalDeviceExternalMemoryHostPropertiesEXT;
    };
);
