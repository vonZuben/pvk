//! For working with heterogeneous collections
//!
//! This crate has some similarity with [frunk](https://docs.rs/frunk/latest/frunk/), but makes different design decisions. Mainly, this
//! crate tries to be more generic.
//!
//! The main use is to create an hlist using [hlist!]. You can then operate on the contained types using the methods provided by
//! the traits in [higher_order], which provide higher order functionality like map and fold. Users can implement [higher_order::FuncMut] in
//! order to implement their own functions which are generic over the different types in the hlist.
//!
//! An hlist is a chain of nested [Cons]. However, in this documentation, we will just call it by a fake type name `Hlist` to make it easier.
//! e.g. Given an hlist of type `Cons<A, Cons<B, Cons<C, End>>>`, we will just refer to it as `Hlist[[A, B, C]]` (using double braces to
//! help point out that it is not standard syntax).
//!
//! ## Example
//!
//! ```
//! use krs_hlist::hlist;
//! use krs_hlist::higher_order::prelude::*;
//!
//! struct Print;
//!
//! impl<D: std::fmt::Display> FuncMut<D> for Print {
//!     type Output = ();
//!     fn call_mut(&mut self, input: D) {
//!         println!("{input}");
//!     }
//! }
//!
//! fn main() {
//!     let list = hlist!(1, "hello", true);
//!     /* will print:
//!         1
//!         hello
//!         true
//!     */
//!     list.for_each(Print);
//! }
//!
//! ```

#![warn(missing_docs)]

use std::ops::Add;
use std::fmt;

mod const_utils;
pub mod higher_order;

pub use const_utils::Comparator;

pub use krs_hlist_pm::{hlist, hlist_ty};

/// Represents a a generic hlist
///
/// A properly constructed hlist (nested chain of [Cons] ending with [End]) will implement this trait automatically
pub trait Hlist {
    /// Head of hlist
    type Head;
    /// Tail of hlist, which itself must be a valid hlist
    type Tail: Hlist;
    /// number of items in the hlist
    const LEN: usize;
}

/// The main building block of hlist
///
/// An hlist is a nested chain of this type, where the last `tail` is set to [End]. e.g. `Cons<A, Cons<B, Cons<C, End>>>`.
/// Normally, you only build an hlist with [hlist!].
///
/// *Note* this type is repr(C) at this time in order to work soundly with the current implementation of [Contains], but it
/// is not clear if this will be maintained.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Cons<H, T> {
    /// This holds that actual data for each spot in the hlist
    pub head: H,
    /// This hold the next part of the list, which should be another 'Cons' or [End]
    pub tail: T,
}

impl<H, T> Cons<H, T> {
    /// Make a new hlist by prepending a head on to another hlist
    pub fn new(head: H, tail: T) -> Self where T: Hlist {
        Self { head, tail }
    }
    /// Append an item to the hlist
    ///
    /// The corresponding method on [End] is for adding items to an empty list.
    pub fn append<I>(self, item: I) -> <Self as Add<Cons<I, End>>>::Output where Self: Add<Cons<I, End>> {
        self + Cons { head: item, tail: End }
    }
}

impl<H, T: Hlist> Hlist for Cons<H, T> {
    type Head = H;
    type Tail = T;
    const LEN: usize = T::LEN + 1;
}

trait DebugFormatHlistTail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<H: fmt::Debug, T: DebugFormatHlistTail> DebugFormatHlistTail for Cons<H, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ", {:?}", self.head)?;
        self.tail.fmt(f)
    }
}

impl<H: fmt::Debug, T: DebugFormatHlistTail> fmt::Debug for Cons<H, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hlist[{:?}", self.head)?;
        self.tail.fmt(f)?;
        write!(f, "]")
    }
}

/// Mark the end of an hlist
///
/// The last `tail` in an hlist should be set with this
#[derive(Clone, Copy, Default)]
pub struct End;

impl End {
    /// Append an item to the hlist
    ///
    /// The corresponding method on [Cons] is for adding items to a non-empty list.
    pub fn append<I>(self, item: I) -> <Self as Add<Cons<I, End>>>::Output where Self: Add<Cons<I, End>> {
        self + Cons { head: item, tail: End }
    }
}

impl Hlist for End {
    type Head = End;
    type Tail = End;
    const LEN: usize = 0;
}

impl fmt::Debug for End {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hlist[]")
    }
}

impl DebugFormatHlistTail for End {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

/// This allows adding different hlists together
///
/// If you want to add an individual item to a list, see [append](Cons::append).
impl<H, T, RHS> Add<RHS> for Cons<H, T>
where
    T: Add<RHS>,
    RHS: Hlist,
{
    type Output = Cons<H, T::Output>;
    fn add(self, rhs: RHS) -> Self::Output {
        Cons {
            head: self.head,
            tail: self.tail + rhs,
        }
    }
}

/// This allows adding different hlists together
///
/// If you want to add an individual item to a list, see [append](End::append).
impl<RHS: Hlist> Add<RHS> for End {
    type Output = RHS;
    fn add(self, rhs: RHS) -> Self::Output {
        rhs
    }
}

/// Represent if an `Hlist` contains a specific type
///
/// Looks in `Hlist` for `T` using comparator `C`
///
pub trait Contains<T, C> {
    /// offset in bytes from start of `Hlist` to where `T` can be found, if T is contained
    const OFFSET: Option<isize>;

    // const HAS: bool = Self::OFFSET.is_some();  // for use with "associated_const_equality"
}

impl<T, C, L: const_utils::Searchable<T, C>> Contains<T, C> for L {
    const OFFSET: Option<isize> = const_utils::get_cons_offset::<T, C, L>();
}

/// Get type T, search using comparator C
///
/// #TODO
/// This trait is implemented for things that Contain T.
/// This is based on the [Contains] trait. However, currently, we cannot confirm that
/// T is actually contained.
///
/// If "associated_const_equality" becomes available, it should be used to check at
/// compile time that T is actually contained
pub trait Get<T, C> {
    /// offset in bytes from start of `Hlist` to where `T` can be found
    const OFFSET: isize;
    /// get `&T` at [OFFSET](Contains::OFFSET)
    fn get(&self) -> &T;
    /// get `&mut T` at [OFFSET](Contains::OFFSET)
    fn get_mut(&mut self) -> &mut T;
}

impl<T, C, L: Contains<T, C>> Get<T, C> for L {
    const OFFSET: isize = const_utils::expect_some(<L as Contains<T, C>>::OFFSET, "error: type T is not in hlist");
    fn get(&self) -> &T {
        let t_ptr: *const T = (self as *const Self).cast();
        unsafe { &*t_ptr.offset(<Self as Get<T, C>>::OFFSET) }
    }
    fn get_mut(&mut self) -> &mut T {
        let t_ptr: *mut T = (self as *mut Self).cast();
        unsafe { &mut *t_ptr.offset(<Self as Get<T, C>>::OFFSET) }
    }
}

// Get T using C as a Comparator
// if "associated_const_equality" becomes stable
// trait Get<T, C> : Contains<T, C, HAS==true> {
//     fn get(&self) -> &T;
//     fn get_mut(&self) -> &mut T;
// }

impl<Head: Iterator, Tail: Iterator> Iterator for Cons<Head, Tail> {
    type Item = Cons<Head::Item, Tail::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        Cons { head: self.head.next()?, tail: self.tail.next()? }.into()
    }
}

impl Iterator for End {
    type Item = End;
    fn next(&mut self) -> Option<Self::Item> {
        End.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    trait MyId {
        const ID: usize;
    }

    struct Comp;

    unsafe impl<A: MyId, B: MyId> Comparator<A, B> for Comp {
        const EQUAL: bool = A::ID == B::ID;
    }

    impl MyId for i32 {
        const ID: usize = 1;
    }

    impl MyId for u32 {
        const ID: usize = 2;
    }

    impl MyId for i8 {
        const ID: usize = 3;
    }

    fn get_u32(list: &impl Contains<u32, Comp>) {
        let _ = list.get();
    }

    #[test]
    fn test_contains() {
        let list = Cons{head: 5u32, tail: Cons{head: 10i32, tail: End}};
        get_u32(&list);
    }
}
