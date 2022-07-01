use std::marker::PhantomData;
use std::ops::BitOr;

use krs_hlist::higher_order::prelude::*;

use crate::to_tokens::*;
use super::{ApplyPrepareQuote, ApplyToTokens};

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

// pub trait RefLike : Copy {}

// impl<T: ?Sized> RefLike for &T {}

#[derive(Clone, Copy, Debug)]
pub struct ToPrepare<R, KIND>(R, PhantomData<KIND>);

macro_rules! to_prepare_trait {
    ( $trait_name:ident -> $kind:ident <$($generics:tt),*> for $from:ty => $to:ty { where $($applicable:tt)* } {
        type Output = $out:ty;
        |$this:ident| $prepare:expr;
        $iter:ident
    }) => {

        #[derive(Copy, Clone, Debug)]
        pub struct $kind;

        pub trait $trait_name<R> {
            fn as_to_prepare(self) -> ToPrepare<R, $kind>;
        }

        impl<$($generics),*> $trait_name<$to> for $from where $($applicable)* {
            fn as_to_prepare(self) -> ToPrepare<$to, $kind> {
                ToPrepare(self, PhantomData)
            }
        }

        impl<$($generics),*> PrepareQuote for ToPrepare<$to, $kind> where $($applicable)* {
            type Output = $out;
            fn prepare_quote(self) -> Self::Output {
                (|$this: Self| $prepare)(self)
            }
        }

        impl<R> BitOr<&ToPrepare<R, $kind>> for NoIter {
            type Output = $iter;
            fn bitor(self, _rhs: &ToPrepare<R, $kind>) -> Self::Output {
                $iter
            }
        }
    }
}

pub mod prepare_different_types {
    use super::*;

    to_prepare_trait!(PrepareRef -> Ref <'a, T> for &'a T => &'a T { where T: ToTokens } {
        type Output = std::iter::Repeat<&'a T>;
        |this| std::iter::repeat(this.0);
        NoIter
    });

    to_prepare_trait!(PrepareSlice -> Slice <'a, T> for &&'a [T] => &'a [T] { where T: ToTokens } {
        type Output = <&'a [T] as IntoIterator>::IntoIter;
        |this| this.0.into_iter();
        HasIter
    });

    to_prepare_trait!(PrepareRefIntoIter -> RefIntoIter <'a, T> for &'a T => &'a T { where &'a T: IntoIterator, <&'a T as IntoIterator>::Item : ToTokens } {
        type Output = <&'a T as IntoIterator>::IntoIter;
        |this| this.0.into_iter();
        HasIter
    });

    to_prepare_trait!(PrepareCloneIntoIter -> CloneIntoIter <'a, T> for &&'a T => &'a T { where T: Clone + IntoIterator, <T as IntoIterator>::Item : ToTokens } {
        type Output = <T as IntoIterator>::IntoIter;
        |this| this.0.clone().into_iter();
        HasIter
    });
}

pub struct NoIter;
pub struct HasIter;

impl<R, KIND> BitOr<&ToPrepare<R, KIND>> for HasIter {
    type Output = HasIter;
    fn bitor(self, _rhs: &ToPrepare<R, KIND>) -> Self::Output {
        HasIter
    }
}

impl<R> BitOr<&super::InnerRep<R>> for HasIter {
    type Output = HasIter;
    fn bitor(self, _rhs: &super::InnerRep<R>) -> Self::Output {
        HasIter
    }
}

impl<R, T> BitOr<&super::InnerRepWithSeparator<R, T>> for HasIter {
    type Output = HasIter;
    fn bitor(self, _rhs: &super::InnerRepWithSeparator<R, T>) -> Self::Output {
        HasIter
    }
}

impl<R> BitOr<&super::InnerRep<R>> for NoIter {
    type Output = NoIter;
    fn bitor(self, _rhs: &super::InnerRep<R>) -> Self::Output {
        NoIter
    }
}

impl<R, T> BitOr<&super::InnerRepWithSeparator<R, T>> for NoIter {
    type Output = NoIter;
    fn bitor(self, _rhs: &super::InnerRepWithSeparator<R, T>) -> Self::Output {
        NoIter
    }
}

pub struct FoldHasIter;

impl<A, B> FuncMut<(A, B)> for FoldHasIter
where
    A: BitOr<B>,
{
    type Output = A::Output;
    fn call_mut(&mut self, i: (A, B)) -> Self::Output {
        i.0 | i.1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HlistWrapper<C>(C);

impl<C> HlistWrapper<C> {
    pub fn new(cons: C) -> Self {
        Self(cons)
    }
}

impl<'a, C: ForEach<ApplyPrepareQuote>> PrepareQuote for &'a HlistWrapper<C> {
    type Output = HlistWrapper<ForEachOut<'a, C, ApplyPrepareQuote>>;
    fn prepare_quote(self) -> Self::Output {
        HlistWrapper::new(self.0.for_each(ApplyPrepareQuote))
    }
}

impl<C: for<'t> ForEach<ApplyToTokens<'t>>> ToTokens for HlistWrapper<C> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.for_each(ApplyToTokens(tokens));
    }
}

impl<C: Iterator> Iterator for HlistWrapper<C> {
    type Item = HlistWrapper<C::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        HlistWrapper::new(self.0.next()?).into()
    }
}