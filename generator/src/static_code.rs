/// I call this "static" code because it is not dynamically generated from vk.xml
///
/// it is basic utility code used by other code that is generated based on vk.xml
pub struct StaticCode;

impl krs_quote::ToTokens for StaticCode {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        tokens.push(stringify!(
            pub(crate) struct DbgStringAsDisplay<'a>(pub(crate) &'a str);

            impl std::fmt::Debug for DbgStringAsDisplay<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    <str as std::fmt::Display>::fmt(&self.0, f)
                }
            }

            pub type PFN_vkVoidFunction = unsafe extern "system" fn() -> ();

            /// raw c string that is guaranteed to be a valid string for use in Vulkan context
            ///
            /// this is only constructed in the vulkan code generator for strings in vk.xml in specific situations
            #[derive(Clone, Copy)]
            #[repr(transparent)]
            pub struct VkStrRaw(*const std::ffi::c_char);

            impl VkStrRaw {
                pub unsafe fn new(ptr: *const std::ffi::c_char) -> Self {
                    Self(ptr)
                }

                pub fn as_ptr(self) -> *const std::ffi::c_char {
                    self.0
                }
            }

            macro_rules! vk_bitflags_wrapped {
                ($name: ident, $ty_name: ty) => {

                    impl $name {
                        #[inline]
                        pub const fn empty() -> $name {
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
                        pub const unsafe fn from_raw(x: $ty_name) -> Self { $name(x) }

                        #[inline]
                        pub const fn as_raw(self) -> $ty_name { self.0 }

                        #[inline]
                        pub const fn is_empty(self) -> bool {
                            self.eq(Self::empty())
                        }

                        #[doc = r" Returns whether `other` is a subset of `self`"]
                        #[inline]
                        pub const fn contains(self, other: $name) -> bool {
                            self.and(other).eq(other)
                        }

                        #[doc = r" Returns whether `self` includes any bits from `other`"]
                        #[inline]
                        pub const fn any_of(self, other: $name) -> bool {
                            !self.and(other).eq(Self::empty())
                        }

                        #[doc = r" Returns whether `self` includes bits only from `other`"]
                        #[inline]
                        pub const fn subset_of(self, other: $name) -> bool {
                            self.0 | other.0 == other.0
                        }

                        /// compare equal for const
                        #[inline]
                        pub const fn eq(self, other: $name) -> bool {
                            self.0 == other.0
                        }

                        /// bitwise AND for const
                        #[inline]
                        pub const fn and(self, other: $name) -> Self {
                            Self(self.0 & other.0)
                        }

                        /// bitwise OR for const
                        #[inline]
                        pub const fn or(self, other: $name) -> Self {
                            Self(self.0 | other.0)
                        }

                        /// return the number of set bits
                        #[inline]
                        pub const fn count_bits(self) -> u32 {
                            self.0.count_ones()
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
