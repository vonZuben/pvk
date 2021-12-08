pub mod instance {

    macro_rules! use_instance_extension_commands {
        ( $($version:ident),* ) => {
            $( $version!( @INSTANCE_COMMANDS make_commands_type $version => ); )*
        };
    }

    use_all_vulkan_extension_names!(use_instance_extension_commands);

}

pub mod device {

    macro_rules! use_device_extension_commands {
        ( $($version:ident),* ) => {
            $( $version!( @DEVICE_COMMANDS make_commands_type $version => ); )*
        };
    }

    use_all_vulkan_extension_names!(use_device_extension_commands);

}

macro_rules! make_extention_implementor {
    ( $m_name:ident => $($iex:ident),* ; $($dex:ident),* ) => {
        #[macro_export]
        macro_rules! $m_name {
            ( @INSTANCE_COMMANDS $name:ident ) => {
                $crate::impl_fptr_traits!($name => $($iex),*);
            };
            ( @DEVICE_COMMANDS $name:ident ) => {
                $crate::impl_fptr_traits!($name => $($dex),*);
            };
        }
    };
}

macro_rules! use_instance_and_device_extension_commands {
    ( @EXTENSIONS $($extension:ident),* ) => {
        $(
            $extension!( @ALL_COMMANDS use_instance_and_device_extension_commands @INNER $extension => );
        )*
    };
    ( @INNER $extension:ident => $($iex:ident),* ; $($dex:ident),* ) => {
        make_extention_implementor!( $extension => $($iex),* ; $($dex),* );
    };
    () => {
        use_all_vulkan_extension_names!(use_instance_and_device_extension_commands @EXTENSIONS);
    };
}

use_instance_and_device_extension_commands!();