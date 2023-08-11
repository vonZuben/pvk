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

use crate::to_tokens::ToTokens;

#[doc(hidden)]
pub struct IsIter;
#[doc(hidden)]
pub struct IsIntoIter;
#[doc(hidden)]
pub struct IsToTokens;
#[doc(hidden)]
pub struct InputWrapper<'a, I: ?Sized, Kind>(&'a I, std::marker::PhantomData<Kind>);
#[doc(hidden)]
pub trait GetTokenIter {
    type Item: ToTokens;
    type TokenIter: Iterator<Item = Self::Item>;
    fn get_token_iter(&self) -> Self::TokenIter;
}

impl<'a, I: Clone + Iterator> GetTokenIter for InputWrapper<'a, I, IsIter> where I::Item: ToTokens {
    type Item = I::Item;

    type TokenIter = I;

    fn get_token_iter(&self) -> Self::TokenIter {
        self.0.clone()
    }
}

impl<'a, I: ?Sized, T: ToTokens, Iter: Iterator<Item = T>> GetTokenIter for InputWrapper<'a, I, IsIntoIter> where &'a I: IntoIterator<Item = T, IntoIter = Iter> {
    type Item = T;

    type TokenIter = Iter;

    fn get_token_iter(&self) -> Self::TokenIter {
        self.0.into_iter()
    }
}

impl<'a, I: ToTokens> GetTokenIter for InputWrapper<'a, I, IsToTokens> {
    type Item = &'a I;

    type TokenIter = std::iter::Repeat<&'a I>;

    fn get_token_iter(&self) -> Self::TokenIter {
        std::iter::repeat(self.0)
    }
}

#[doc(hidden)]
pub trait IterInput {
    fn input<'a>(&'a self) -> InputWrapper<'a, Self, IsIter>;
}

impl<T: ?Sized> IterInput for T where T: Clone + Iterator {
    fn input<'a>(&'a self) -> InputWrapper<'a, Self, IsIter> {
        InputWrapper(self, Default::default())
    }
}

#[doc(hidden)]
pub trait IntoIterInput<'a, T: ?Sized> {
    fn input(&self) -> InputWrapper<'a, T, IsIntoIter>;
}

impl<'a, T: ?Sized> IntoIterInput<'a, T> for &'a T where Self: IntoIterator {
    fn input(&self) -> InputWrapper<'a, T, IsIntoIter> {
        InputWrapper(*self, Default::default())
    }
}

#[doc(hidden)]
pub trait TokenInput {
    fn input<'a>(&'a self) -> InputWrapper<'a, Self, IsToTokens>;
}

impl<T: ?Sized> TokenInput for T where T: ToTokens {
    fn input<'a>(&'a self) -> InputWrapper<'a, Self, IsToTokens> {
        InputWrapper(self, Default::default())
    }
}