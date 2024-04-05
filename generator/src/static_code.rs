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

            use std::fmt;

            // =================VkVersion===========================
            #[repr(transparent)]
            pub struct VkVersion(u32);

            impl VkVersion {
                pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
                    Self::new_with_variant(0, major, minor, patch)
                }
                pub const fn from_triple((major, minor, patch): (u32, u32, u32)) -> Self {
                    Self::new(major, minor, patch)
                }
                pub const fn new_with_variant(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
                    Self((variant << 29) | (major << 22) | (minor << 12) | (patch))
                }
                pub const fn parts(&self) -> (u32, u32, u32) {
                    let parts = self.parts_with_variant();
                    (parts.1, parts.2, parts.3)
                }
                pub const fn parts_with_variant(&self) -> (u32, u32, u32, u32) {
                    (
                        self.0 >> 29,
                        (self.0 >> 22) & 0x7F,
                        (self.0 >> 12) & 0x3FF,
                        self.0 & 0xFFF,
                    )
                }
                pub const fn raw(&self) -> u32 {
                    self.0
                }
                pub const unsafe fn from_raw(raw: u32) -> Self {
                    Self(raw)
                }
            }

            impl fmt::Debug for VkVersion {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    <Self as fmt::Display>::fmt(&self, f)
                }
            }

            impl fmt::Display for VkVersion {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let (variant, major, minor, patch) = self.parts_with_variant();
                    if variant != 0 {
                        write!(f, "{major}.{minor}.{patch} - variant: {variant}")
                    } else {
                        write!(f, "{major}.{minor}.{patch}")
                    }
                }
            }

            impl From<(u32, u32, u32)> for VkVersion {
                fn from((major, minor, patch): (u32, u32, u32)) -> Self {
                    Self::new(major, minor, patch)
                }
            }

            impl std::cmp::PartialEq for VkVersion {
                fn eq(&self, other: &Self) -> bool {
                    // check if equal without the variant
                    VkVersion::from_triple(self.parts()).0 == VkVersion::from_triple(other.parts()).0
                }
            }

            impl std::cmp::PartialOrd for VkVersion {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    // compare versions without the variant
                    VkVersion::from_triple(self.parts())
                        .0
                        .partial_cmp(&VkVersion::from_triple(other.parts()).0)
                }
            }

            #[cfg(test)]
            mod test {
                use super::VkVersion;

                #[test]
                fn test_no_variant() {
                    let v = VkVersion::new(1, 2, 3);

                    println!("{v}");
                    println!("{v:?}");
                }

                #[test]
                fn test_with_variant() {
                    let v = VkVersion::new_with_variant(1, 1, 2, 3);

                    println!("{v}");
                    println!("{v:?}");
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

                        #[inline]
                        pub const fn is_not_empty(self) -> bool {
                            !self.is_empty()
                        }

                        #[doc = r" Returns true if `other` is a subset of `self`; always false if other is empty"]
                        #[inline]
                        pub const fn contains(self, other: $name) -> bool {
                            other.subset_of(self)
                        }

                        #[doc = r" Returns true if `self` includes any bits from `other`"]
                        #[inline]
                        pub const fn any_of(self, other: $name) -> bool {
                            !self.and(other).eq(Self::empty())
                        }

                        #[doc = r" Returns true if `self` includes bits only from `other`; always false if self is empty"]
                        #[inline]
                        pub const fn subset_of(self, other: $name) -> bool {
                            self.or(other).eq(other) && self.is_not_empty()
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
