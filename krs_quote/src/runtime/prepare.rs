use std::marker::PhantomData;

use krs_hlist::higher_order_prelude::*;

use crate::to_tokens::*;
use super::ApplyPrepareQuote;

pub trait PrepareQuote {
    type Output;
    fn prepare_quote(self) -> Self::Output;
}

impl PrepareQuote for RawToken {
    type Output = std::iter::Repeat<RawToken>;
    fn prepare_quote(self) -> Self::Output {
        std::iter::repeat(self.clone())
    }
}

impl<P: PrepareQuote + Copy> PrepareQuote for &P {
    type Output = P::Output;

    fn prepare_quote(self) -> Self::Output {
        (*self).prepare_quote()
    }
}

#[derive(Debug)]
pub struct ToPrepare<'a, T: ?Sized, KIND>(&'a T, PhantomData<KIND>);

impl<'a, T: ?Sized, KIND> Copy for ToPrepare<'a, T, KIND> {}

impl<'a, T: ?Sized, KIND> Clone for ToPrepare<'a, T, KIND> {
    fn clone(&self) -> Self {
        *self
    }
}

macro_rules! to_prepare_trait {
    ( $trait_name:ident -> $kind:ident <$life:lifetime, $ty:tt> for $apply:ty { where $($applicable:tt)* } {
        type Output = $out:ty;
        |$this:ident| $prepare:expr
    }) => {

        #[derive(Copy, Clone, Debug)]
        pub struct $kind;

        pub trait $trait_name<$life, $ty: ?Sized> {
            fn as_to_prepare(&$life self) -> ToPrepare<$life, $ty, $kind>;
        }

        impl<$life, $ty: ?Sized> $trait_name<$life, $ty> for $apply where $($applicable)* {
            fn as_to_prepare(&$life self) -> ToPrepare<$life, $ty, $kind> {
                ToPrepare(self, PhantomData)
            }
        }

        impl<$life, $ty: ?Sized> PrepareQuote for ToPrepare<$life, $ty, $kind> where $($applicable)* {
            type Output = $out;
            fn prepare_quote(self) -> Self::Output {
                (|$this: Self| $prepare)(self)
            }
        }
    }
}

pub mod prepare_different_types {
    use super::*;
    to_prepare_trait!(PrepareRef -> Ref <'a, T> for T { where T: ToTokens } {
        type Output = std::iter::Repeat<&'a T>;
        |this| std::iter::repeat(this.0)
    });

    to_prepare_trait!(PrepareRefIntoIter -> RefIntoIter <'a, T> for T { where for<'t> &'t T: IntoIterator, for<'t> <&'t T as IntoIterator>::Item : ToTokens } {
        type Output = <&'a T as IntoIterator>::IntoIter;
        |this| this.0.into_iter()
    });

    to_prepare_trait!(PrepareCloneIntoIter -> CloneIntoIter <'a, T> for &T { where T: Clone + IntoIterator, <T as IntoIterator>::Item : ToTokens } {
        type Output = <T as IntoIterator>::IntoIter;
        |this| this.0.clone().into_iter()
    });
}

pub struct PrepareConsWrapper<C>(pub C);

impl<'a, C: ForEach<ApplyPrepareQuote>> PrepareQuote for &'a PrepareConsWrapper<C> {
    type Output = ForEachOut<'a, C, ApplyPrepareQuote>;
    fn prepare_quote(self) -> Self::Output {
        self.0.for_each(ApplyPrepareQuote)
    }
}
