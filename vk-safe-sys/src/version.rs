macro_rules! impl_version {
    ( $version:ident , $version_tuple:tt ) => {
        impl super::Version for $version {
            const VersionTuple: (u32, u32, u32) = $version_tuple;
            fn load(f: impl $crate::commands::FunctionLoader) -> Result<Self, $crate::commands::LoadError> {
                Self::load(f)
            }
        }
    }
}

pub trait Version : Sized {
    const VersionTuple: (u32, u32, u32);
    fn load(f: impl crate::commands::FunctionLoader) -> Result<Self, crate::commands::LoadError>;
}

pub mod instance {

    macro_rules! use_instance_feature_commands {
        ( $($version:ident => $version_tuple:tt),* ) => {
            $( 
                $version!( @INSTANCE make_commands_type $version => );
                impl_version!( $version , $version_tuple );
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
                impl_version!( $version , $version_tuple );
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
                impl_version!( $version , $version_tuple );
            )*
        };
    }

    use_all_vulkan_version_names!(use_device_feature_commands);

}