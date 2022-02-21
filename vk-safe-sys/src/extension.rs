
pub struct LoadNone;
pub struct Load(*const std::os::raw::c_char);

pub trait Extension {
    /// This is intended to be an Hlist representing all other extension prerequisists for this extension
    type Require;

    /// Represent if this extension needs to load (i.e. some implementors of this trait only represent additional optional commands which otherwise require a base extension)
    type Load;

    /// What to load if any
    const LoadThis: Self::Load;
}

macro_rules! impl_extension {
    // when has no load
    ( $name:ident ; $($require:ident),* ) => {
        impl super::Extension for $name {
            type Require = hlist_ty!($($require),*);
            type Load = super::LoadNone;
            const LoadThis: Self::Load = super::LoadNone;
        }
    };
    // when has laod
    ( $name:ident $load:literal ; $($require:ident),* ) => {
        impl super::Extension for $name {
            type Require = hlist_ty!($($require),*);
            type Load = super::Load;
            const LoadThis: Self::Load = super::Load(concat!($load, "\0").as_ptr().cast());
        }
    }
}

pub mod instance {

    macro_rules! use_instance_extensions {
        ( $($ex:ident),* ) => {
            $( $ex!( @KIND use_instance_extensions @INNER $ex ); )*
        };
        ( @INNER $ex:ident INSTANCE ) => {
            $ex!( @ALL_COMMANDS make_commands_type $ex => );
            $ex!( @REQUIRE impl_extension $ex );
        };
        ( @INNER $ex:ident DEVICE ) => {
        };
    }

    use_all_vulkan_extension_names!(use_instance_extensions);

}

pub mod device {

    use super::instance::*;

    macro_rules! use_device_extensions {
        ( $($ex:ident),* ) => {
            $( $ex!( @KIND use_device_extensions @INNER $ex ); )*
        };
        ( @INNER $ex:ident DEVICE ) => {
            $ex!( @ALL_COMMANDS make_commands_type $ex => );
            $ex!( @REQUIRE impl_extension $ex );
        };
        ( @INNER $ex:ident INSTANCE ) => {
        };
    }

    use_all_vulkan_extension_names!(use_device_extensions);

}

macro_rules! make_extention_implementor {
    ( $m_name:ident => $($command:ident),* ) => {
        #[macro_export]
        macro_rules! $m_name {
            ( $name:ident ) => {
                $crate::impl_fptr_traits!($name => $($command),*);
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
    ( @INNER $extension:ident => $($command:ident),* ) => {
        make_extention_implementor!( $extension => $($command),* );
    };
    () => {
        use_all_vulkan_extension_names!(use_instance_and_device_extension_commands @EXTENSIONS);
    };
}

use_instance_and_device_extension_commands!();