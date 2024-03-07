use super::*;

pub struct Delegate;

impl<I: Variants> ToTokensDelegate<I> for Delegate {
    fn delegate_to_tokens(params: &Properties<I>, tokens: &mut TokenStream) {
        let variants = params.variants.clone();

        krs_quote_with!(tokens <-
            #[macro_export]
            macro_rules! queue_capabilities {
                ( $vis:vis $target:ident : $($queue_capability:ident),+ $(,)? ) => {
                    $vis struct $target;
                    impl $crate::queue_capability::QueueCapability for $target {
                        const CAPABILITY: $crate::QueueFlags = $crate::QueueFlags::empty()$( .or($crate::QueueFlags::$queue_capability) )+ ;
                    }
                    $(
                        impl $crate::queue_capability::$queue_capability for $target {}
                    )+
                }
            }

            pub mod queue_capability {
                pub trait QueueCapability {
                    const CAPABILITY: super::QueueFlags;
                }
                {@* pub trait {@variants} : QueueCapability {} }
            }
        );
    }
}
