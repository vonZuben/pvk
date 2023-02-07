
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
