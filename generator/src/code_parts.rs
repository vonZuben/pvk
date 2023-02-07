/// Provide a list of generated code parts for other macros as a list of comma separated identifiers
///
/// $run : macro which can parse a list of comma separated identifiers at the end of the input tokens
/// ($semi?) : if $run will output an 'item', then put a semi colon here for proper syntax
/// $pass* : the rest of the input tokens are passed to the $run macro before the list of module names
macro_rules! code_parts {
    ( $run:ident ($($semi:tt)?) $($pass:tt)* ) => {
        $run!( $($pass)*
            util_code,
            vulkan_traits,
            c_type_defs,
            bitmasks,
            structs,
            unions,
            handles,
            enumerations,
            enum_variants,
            function_pointers,
            constants,
            commands,
            versions,
            extensions,
            aliases,
        )$($semi)?
    };
}
