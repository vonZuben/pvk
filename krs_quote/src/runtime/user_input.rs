//! This module provides a way to automatically deal with different types of user inputs
//! when using the provided quote macro, we want the user to be able to pass straight ToToken
//! implementors, things that can iterate over ToToken implementors repeatedly (by cloning the
//! iterator), and things that can be converted into iterators by ref.
//!
//! "repeatably" and "by ref" are important since some input may be interpolated many times in different
//! repetition expressions (i.e. inside {@*}).
//!
//! The trick is probably a rust anti pattern, but it should work well for the assumptions I make:
//! 1. If something is ToTokens, it is not an iterator
//! 2. vice versa for iterators
//! 3. similar for IntoIterators, but 'Option' is an exception (we use "autoref" to deal with this, see below)
//!
//! 'IterInput' and 'TokenInput' are at the same "ref level" which accept a &T depending T: ToTokens, or T: Clone + Iterator
//! there should be no overlapping implementations from my assumptions
//!
//! 'IntoIterInput' is on the "next ref level", where if neither of the above implementations exist, "autoref" will kick in,
//! and we hopefully find an implementation.
//!
//! If none of the above implementations exist, the user input is probably not valid.
//!
//! This is all internal implementations details, and these docs are just for internal use.

use std::iter::Repeat;

use crate::to_tokens::{ToTokens, IterWrapper};

#[doc(hidden)]
pub trait IterInput {
    fn input(&self) -> IterWrapper<Self> where Self: Sized;
}

impl<T: ?Sized> IterInput for T where T: Clone + Iterator {
    fn input(&self) -> IterWrapper<Self> {
        IterWrapper(self.clone())
    }
}

#[doc(hidden)]
pub trait IntoIterInput {
    type Iter;
    fn input(&self) -> IterWrapper<Self::Iter>;
}

impl<'a, T: ?Sized> IntoIterInput for &'a T where Self: IntoIterator {
    type Iter = <Self as IntoIterator>::IntoIter;
    fn input(&self) -> IterWrapper<Self::Iter> {
        IterWrapper(self.into_iter())
    }
}

#[doc(hidden)]
pub trait TokenInput {
    fn input<'a>(&'a self) -> IterWrapper<Repeat<&'a Self>>;
}

impl<T: ?Sized> TokenInput for T where T: ToTokens {
    fn input<'a>(&'a self) -> IterWrapper<Repeat<&'a Self>> {
        IterWrapper(std::iter::repeat(self))
    }
}