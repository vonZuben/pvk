pub mod instance {

    macro_rules! use_instance_feature_commands {
        ( $($version:ident),* ) => {
            $( $version!( @INSTANCE make_commands_type $version => ); )*
        };
    }

    use_all_vulkan_version_names!(use_instance_feature_commands);

}

pub mod device {

    macro_rules! use_device_feature_commands {
        ( $($version:ident),* ) => {
            $( $version!( @DEVICE make_commands_type $version => ); )*
        };
    }

    use_all_vulkan_version_names!(use_device_feature_commands);

}