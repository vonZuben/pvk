pub unsafe trait Flags {
    type Type;
    const FLAGS: Self::Type;
    const NOT_FLAGS: Self::Type;
}

pub unsafe trait Flag<F>: Flags {}

pub unsafe trait NotFlag<F>: Flags {}

#[macro_export]
macro_rules! flags {
    ( $vis:vis $name:ident : $f_type:ident $( + $has:ident )* $( - $not:ident )* ) => {
        $vis struct $name;

        {
            use vk_safe_sys::flag_types::$f_type;
            $( use $f_type::$has; )*
            $( use $f_type::$not; )*

            unsafe impl $crate::Flags for $name {
                type Type = vk_safe_sys::$f_type;
                const FLAGS: Self::Type = ( Self::Type::empty() $( .or(Self::Type::$has) )* );
                const NOT_FLAGS: Self::Type = ( Self::Type::empty() $( .or(Self::Type::$not) )* );
            }

            $(
                unsafe impl $crate::Flag<$has> for $name {}
            )*

            $(
                unsafe impl $crate::NotFlag<$not> for $name {}
            )*
        }
    };
}
