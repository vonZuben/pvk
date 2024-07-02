macro_rules! unit_error {
    (
        $(#[$($attributes:tt)*])*
        $vis:vis $name:ident
    ) => {
        #[derive(Debug)]
        $(#[$($attributes)*])*
        $vis struct $name;

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }

        impl std::error::Error for $name {}
    };
}

macro_rules! enum_error {
    (
        $(#[$($attributes:tt)*])*
        $vis:vis enum $name:ident {
            $( $variant:ident ),+
            $(,)?
        }
    ) => {
        #[derive(Debug)]
        $(#[$($attributes)*])*
        $vis enum $name {
            $( $variant ),+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }

        impl std::error::Error for $name {}
    };
}
