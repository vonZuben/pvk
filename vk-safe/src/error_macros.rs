macro_rules! unit_error {
    ( $vis:vis $name:ident ) => {
        #[derive(Debug)]
        $vis struct $name;

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }

        impl std::error::Error for $name {}
    };
}
