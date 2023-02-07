use crate::commands;

// macro_rules! impl_version {
//     ( $version:ident , $version_tuple:tt ) => {
//         impl super::Version for $version {
//             const VersionTuple: (u32, u32, u32) = $version_tuple;
//             fn load(f: impl $crate::commands::FunctionLoader) -> Result<Self, $crate::commands::LoadError> {
//                 Self::load(f)
//             }
//         }
//     }
// }

// pub trait Version : Sized {
//     const VersionTuple: (u32, u32, u32);
//     fn load(f: impl commands::FunctionLoader) -> Result<Self, commands::LoadError>;
// }

pub trait Version {
    const VersionTuple: (u32, u32, u32);
    type InstanceCommands : commands::LoadCommands;
    type DeviceCommands : commands::LoadCommands;
    type EntryCommands : commands::LoadCommands;
    fn load_instance_commands(f: impl commands::FunctionLoader) -> Result<Self::InstanceCommands, commands::LoadError> {
        <Self::InstanceCommands as commands::LoadCommands>::load(f)
    }
    fn load_device_commands(f: impl commands::FunctionLoader) -> Result<Self::DeviceCommands, commands::LoadError> {
        <Self::DeviceCommands as commands::LoadCommands>::load(f)
    }
    fn load_entry_commands(f: impl commands::FunctionLoader) -> Result<Self::EntryCommands, commands::LoadError> {
        <Self::EntryCommands as commands::LoadCommands>::load(f)
    }
}

macro_rules! impl_version {
    ( $version:ident , $version_tuple:tt => 
        $($instance_commands:ident),* ; 
        $($device_commands:ident),* ; 
        $($entry_commands:ident),* ) => {
            pub struct $version;
            impl Version for $version {
                const VersionTuple: (u32, u32, u32) = $version_tuple;
                type InstanceCommands = instance::$version;
                type DeviceCommands = device::$version;
                type EntryCommands = entry::$version;
                // type InstanceCommands = hlist_ty!( $($crate::commands::loaders::$instance_commands),* );
                // type DeviceCommands = hlist_ty!( $($crate::commands::loaders::$device_commands),* );
                // type EntryCommands = hlist_ty!( $($crate::commands::loaders::$entry_commands),* );
            }
    }
}

macro_rules! use_feature_commands {
    ( $($version:ident => $version_tuple:tt),* ) => {
        $( 
            $version!( @ALL impl_version $version , $version_tuple => );
        )*
    };
}

use_all_vulkan_version_names!(use_feature_commands);

pub mod instance {

    macro_rules! use_instance_feature_commands {
        ( $($version:ident => $version_tuple:tt),* ) => {
            $( 
                $version!( @INSTANCE make_commands_type $version => );
                // impl_version!( $version , $version_tuple );
                // $version!( @ALL impl_version2 $version , $version_tuple => );
            )*
        };
    }

    use_all_vulkan_version_names!(use_instance_feature_commands);

}

pub mod device {

    macro_rules! use_device_feature_commands {
        ( $($version:ident => $version_tuple:tt),* ) => {
            $( 
                $version!( @DEVICE make_commands_type $version => );
                // impl_version!( $version , $version_tuple );
            )*
        };
    }

    use_all_vulkan_version_names!(use_device_feature_commands);

}

pub mod entry {

    macro_rules! use_device_feature_commands {
        ( $($version:ident => $version_tuple:tt),* ) => {
            $( 
                $version!( @ENTRY make_commands_type $version => );
                // impl_version!( $version , $version_tuple );
            )*
        };
    }

    use_all_vulkan_version_names!(use_device_feature_commands);

}