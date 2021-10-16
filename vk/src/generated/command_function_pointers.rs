
pub type PLACEHOLDER = unsafe extern "system" fn();
pub type PLACEHOLDER2 = unsafe extern "system" fn();

// sample ex
pub type PLACEHOLDER_EX = unsafe extern "system" fn();

macro_rules! use_command_function_pointer_names {
    ( $call:ident $($pass:tt)* ) => {
        $call!( $($pass)* PLACEHOLDER, PLACEHOLDER2, PLACEHOLDER_EX );
    }
}