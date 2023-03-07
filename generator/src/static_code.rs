
/// I call this "static" code because it is not dynamically generated from vk.xml
///
/// it is basic utility code used by other code that is generated based on vk.xml
pub struct StaticCode;

impl krs_quote::ToTokens for StaticCode {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        tokens.push(
        stringify!(
            pub use krs_hlist::hlist_ty;

            pub(crate) struct DbgStringAsDisplay<'a>(pub(crate) &'a str);

            impl std::fmt::Debug for DbgStringAsDisplay<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    <str as std::fmt::Display>::fmt(&self.0, f)
                }
            }

            pub type PFN_vkVoidFunction = unsafe extern "system" fn() -> ();

            macro_rules! vk_bitflags_wrapped {
                ($name: ident, $ty_name: ty) => {

                    impl $name {
                        #[inline]
                        pub unsafe fn empty() -> $name {
                            $name(0)
                        }

                        // this is for supporting taking each bit one at a time
                        // can use for iterating over each bit
                        // create a copy of the bit field with only the lowest active bit
                        // and unset the same bit in the origin
                        // or returns None if no bits set
                        pub(crate) fn take_lowest_bit(&mut self) -> Option<$name> {
                            let lowest_bit = self.0 & self.0.wrapping_neg();
                            if lowest_bit == 0 {
                                None
                            }
                            else {
                                self.0 ^= lowest_bit;
                                Some($name(lowest_bit))
                            }
                        }

                        #[inline]
                        pub unsafe fn from_raw(x: $ty_name) -> Self { $name(x) }

                        #[inline]
                        pub fn as_raw(self) -> $ty_name { self.0 }

                        #[inline]
                        pub fn is_empty(self) -> bool {
                            self == unsafe { $name::empty() }
                        }

                        #[doc = r" Returns whether `other` is a subset of `self`"]
                        #[inline]
                        pub fn contains(self, other: $name) -> bool {
                            self & other == other
                        }
                    }

                    impl ::std::ops::BitOr for $name {
                        type Output = $name;

                        #[inline]
                        fn bitor(self, rhs: $name) -> $name {
                            $name (self.0 | rhs.0 )
                        }
                    }

                    impl ::std::ops::BitOrAssign for $name {
                        #[inline]
                        fn bitor_assign(&mut self, rhs: $name) {
                            *self = *self | rhs
                        }
                    }

                    impl ::std::ops::BitAnd for $name {
                        type Output = $name;

                        #[inline]
                        fn bitand(self, rhs: $name) -> $name {
                            $name (self.0 & rhs.0)
                        }
                    }

                    impl ::std::ops::BitAndAssign for $name {
                        #[inline]
                        fn bitand_assign(&mut self, rhs: $name) {
                            *self = *self & rhs
                        }
                    }

                    impl ::std::ops::BitXor for $name {
                        type Output = $name;

                        #[inline]
                        fn bitxor(self, rhs: $name) -> $name {
                            $name (self.0 ^ rhs.0 )
                        }
                    }

                    impl ::std::ops::BitXorAssign for $name {
                        #[inline]
                        fn bitxor_assign(&mut self, rhs: $name) {
                            *self = *self ^ rhs
                        }
                    }
                }
            }

        ));
    }
}
