use krs_quote::ToTokens;

pub struct BitmaskTraits;

impl ToTokens for BitmaskTraits {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        tokens.push(
        stringify!(
            use krs_hlist::{Cons, End};

            /// Vulkan flags e.g. VkImageUsageFlags
            pub trait VkBitmaskType : Sized {
                /// the underling type of the flags e.g. VkFlags (uint32_t) or VkFlags64 (uint64_t)
                type RawType;
                // type Verifier: VerifyBits;
                fn from_bit_type_list<L: BitList<Self::RawType, Self>>(_: L) -> Self; // + VerifyFlags<Self>
            }

            /// an individual flag bit for a specific Flags type
            pub trait VkFlagBitType {
                /// The Flags type that this bit is for
                type FlagType : VkBitmaskType;
                const FLAG: <Self::FlagType as VkBitmaskType>::RawType;
            }

            /// zero or more bitflags
            ///
            /// SAFETY: must ensure that the FLAGS value is based on valid bits for FlagsType
            ///
            /// this is designed to get around limitations with const evaluation and trait disambiguation
            ///
            /// we need to implements this per raw type which rust can perform BitOr with in const eval
            /// Also, it should be implemented for a specific FlagType
            pub unsafe trait BitList<RawType, FlagsType> {
                const FLAGS: RawType;
            }

            /// This is always safe since it just marks the end of the list and does not contribute the the FlagsType value
            unsafe impl<R: Zero, FlagsType: VkBitmaskType<RawType = R>> BitList<R, FlagsType> for End {
                const FLAGS: R = R::ZERO;
            }

            trait Zero {
                const ZERO: Self;
            }

            #[derive(Clone, Copy)]
            pub struct SetFlags<T>(T);

            macro_rules! impl_bit_types {
                ( $( $ty:ty ),* ) => {
                    $(
                        impl Zero for $ty {
                            const ZERO: Self = 0;
                        }

                        /// This is safe because we ensure that H::FlagType matches the BitList FlagsType
                        /// and we assume T is also safely implements BitList
                        unsafe impl<FlagsType, H, T> BitList<$ty, FlagsType> for Cons<H, T>
                        where
                            FlagsType: VkBitmaskType<RawType = $ty>,
                            H: VkFlagBitType<FlagType = FlagsType>,
                            T: BitList<$ty, FlagsType>,
                        {
                            const FLAGS: <FlagsType as VkBitmaskType>::RawType = H::FLAG | T::FLAGS;
                        }

                        impl SetFlags<$ty> {
                            pub const fn from(t: $ty) -> Self {
                                Self(t)
                            }
                            pub const fn contains(self, other: $ty) -> bool {
                                self.0 & other == other
                            }
                            pub const fn subset_of(self, of: $ty) -> bool {
                                self.0 | of == of
                            }
                        }
                    )*
                };
            }

            // Implement this for all possible bit types
            // although, Vulkan only uses 32 and 64 bit flags at the moment
            impl_bit_types!(u8, u16, u32, u64, u128);
        ));
    }
}