macro_rules! PLACEHOLDER_EX_NAME {
    ( @INSTANCE $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* PLACEHOLDER_EX );
    };
    ( @DEVICE $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* );
    };
    ( @ALL $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* PLACEHOLDER_EX ; );
    };
}

macro_rules! use_all_vulkan_extension_names {
    ( $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* PLACEHOLDER_EX_NAME );
    }
}