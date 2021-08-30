macro_rules! V10 {
    ( @INSTANCE $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* PLACEHOLDER );
    };
    ( @DEVICE $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* PLACEHOLDER2 );
    };
    ( @ALL $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* PLACEHOLDER ; PLACEHOLDER2 );
    };
}

macro_rules! use_all_vulkan_version_names {
    ( $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* V10 );
    }
}