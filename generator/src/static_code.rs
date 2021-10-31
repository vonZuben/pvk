
use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

pub struct StaticCode;

impl ToTokens for StaticCode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(
            macro_rules! vk_bitflags_wrapped {
                ($name: ident, $ty_name: ty) => {

                    impl Default for $name{
                        fn default() -> $name {
                            $name(0)
                        }
                    }

                    impl $name {
                        #[inline]
                        pub fn empty() -> $name {
                            $name(0)
                        }

                        // TODO fix $all
                        //#[inline]
                        //pub fn all() -> $name {
                        //    $name($all)
                        //}

                        #[inline]
                        pub fn from_raw(x: $ty_name) -> Self { $name(x) }

                        #[inline]
                        pub fn as_raw(self) -> $ty_name { self.0 }

                        #[inline]
                        pub fn is_empty(self) -> bool {
                            self == $name::empty()
                        }

                        //#[inline]
                        //pub fn is_all(self) -> bool {
                        //    self & $name::all() == $name::all()
                        //}

                        //#[inline]
                        //pub fn intersects(self, other: $name) -> bool {
                        //    self & other != $name::empty()
                        //}

                        /// Returns whether `other` is a subset of `self`
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

                    //impl ::std::ops::Sub for $name {
                    //    type Output = $name;

                    //    #[inline]
                    //    fn sub(self, rhs: $name) -> $name {
                    //        self & !rhs
                    //    }
                    //}

                    //impl ::std::ops::SubAssign for $name {
                    //    #[inline]
                    //    fn sub_assign(&mut self, rhs: $name) {
                    //        *self = *self - rhs
                    //    }
                    //}

                    //impl ::std::ops::Not for $name {
                    //    type Output = $name;

                    //    #[inline]
                    //    fn not(self) -> $name {
                    //        self ^ $name::all()
                    //    }
                    //}
                }
            }
            use std::os::raw::*;
            pub type RROutput = c_ulong;
            pub type VisualID = c_uint;
            pub type Display = *const c_void;
            pub type Window = c_ulong;
            #[allow(non_camel_case_types)]
            pub type xcb_connection_t = *const c_void;
            #[allow(non_camel_case_types)]
            pub type xcb_window_t = u32;
            #[allow(non_camel_case_types)]
            pub type xcb_visualid_t = *const c_void;
            pub type MirConnection = *const c_void;
            pub type MirSurface = *const c_void;
            pub type HINSTANCE = *const c_void;
            pub type HWND = *const c_void;
            #[allow(non_camel_case_types)]
            pub type wl_display = c_void;
            #[allow(non_camel_case_types)]
            pub type wl_surface = c_void;
            pub type HANDLE = *mut c_void;
            pub type DWORD = c_ulong;
            pub type LPCWSTR = *const u16;
            #[allow(non_camel_case_types)]
            pub type zx_handle_t = u32;
        
            // FIXME: Platform specific types that should come from a library id:0
            // typedefs are only here so that the code compiles for now
            #[allow(non_camel_case_types)]
            pub type SECURITY_ATTRIBUTES = ();
            // Opage types
            pub type ANativeWindow = c_void;
            pub type AHardwareBuffer = c_void;
        
            // NOTE These type are included only for compilation purposes
            // These types should NOT be used because they are no necessarily
            // the correct type definitions (i.e. just c_void by default)
            pub type GgpStreamDescriptor = *const c_void;
            pub type CAMetalLayer = *const c_void;
            pub type GgpFrameToken = *const c_void;
            pub type HMONITOR = *const c_void;
        ).to_tokens(tokens);
    }
}
