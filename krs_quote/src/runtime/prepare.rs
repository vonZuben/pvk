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

pub trait RefLike : Copy {}

impl<T: ?Sized> RefLike for &T {}

#[derive(Debug)]
pub struct ToPrepare<R: RefLike, KIND>(R, PhantomData<KIND>);

impl<R: RefLike, KIND> Copy for ToPrepare<R, KIND> {}

impl<R: RefLike, KIND> Clone for ToPrepare<R, KIND> {
    fn clone(&self) -> Self {
        *self
    }
}

macro_rules! to_prepare_trait {
    ( $trait_name:ident -> $kind:ident <$life:lifetime, $ty:tt> for $from:ty => $to:ty { where $($applicable:tt)* } {
        type Output = $out:ty;
        |$this:ident| $prepare:expr
    }) => {

        #[derive(Copy, Clone, Debug)]
        pub struct $kind;

        pub trait $trait_name<R: RefLike> {
            fn as_to_prepare(self) -> ToPrepare<R, $kind>;
        }

        impl<$life, $ty> $trait_name<$to> for $from where $($applicable)* {
            fn as_to_prepare(self) -> ToPrepare<$to, $kind> {
                ToPrepare(self, PhantomData)
            }
        }

        impl<$life, $ty> PrepareQuote for ToPrepare<$to, $kind> where $($applicable)* {
            type Output = $out;
            fn prepare_quote(self) -> Self::Output {
                (|$this: Self| $prepare)(self)
            }
        }
    }
}

pub mod prepare_different_types {
    use super::*;
    to_prepare_trait!(PrepareRef -> Ref <'a, T> for &'a T => &'a T { where T: ToTokens } {
        type Output = std::iter::Repeat<&'a T>;
        |this| std::iter::repeat(this.0)
    });

    to_prepare_trait!(PrepareSlice -> Slice <'a, T> for &&'a [T] => &'a [T] { where T: ToTokens } {
        type Output = <&'a [T] as IntoIterator>::IntoIter;
        |this| this.0.into_iter()
    });

    to_prepare_trait!(PrepareRefIntoIter -> RefIntoIter <'a, T> for &'a T => &'a T { where &'a T: IntoIterator, <&'a T as IntoIterator>::Item : ToTokens } {
        type Output = <&'a T as IntoIterator>::IntoIter;
        |this| this.0.into_iter()
    });

    to_prepare_trait!(PrepareCloneIntoIter -> CloneIntoIter <'a, T> for &&'a T => &'a T { where T: Clone + IntoIterator, <T as IntoIterator>::Item : ToTokens } {
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
