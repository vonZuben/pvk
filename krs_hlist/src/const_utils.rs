use super::{Cons, End};

/// Determine how to compare two types
///
/// This trait should be implemented by the crate user for a type
/// that represents how to compare A and B in a constant context.
///
/// It is unsafe to implement because it is relied on to accurately find a
/// specific type in an hlist. If the comparator inaccurately reports two
/// different types as equal, then it will lead to dereferencing pointers
/// to the wrong type.
///
/// ```
/// use krs_hlist::Comparator;
///
/// // This is an implementation detail for the user
/// // in this example, we simply assign each type we want to have a unique ID
/// // It is unsafe sine the ID must be unique
/// unsafe trait Id {
///     const ID: usize;
/// }
///
/// // The comparator simply compares the IDs of each type
/// struct Comp;
/// unsafe impl<A: Id, B: Id> Comparator<A, B> for Comp {
///     const EQUAL: bool = A::ID == B::ID;
/// }
/// ```
pub unsafe trait Comparator<A: ?Sized, B: ?Sized> {
    /// True if A and B ase equal types, False otherwise
    const EQUAL: bool;
}

/// This is so every possible comparator can check against [End]
/// since every hlist should end with `End`.
unsafe impl<T: ?Sized, C> Comparator<T, End> for C {
    const EQUAL: bool = false;
}

/// Compare `Self` with `T` using a comparator C
///
/// Internal detail. User should only use [Comparator]
pub trait CompareWith<T, C> {
    const EQUAL: bool;
}

impl<A, B, C: Comparator<A, B>> CompareWith<A, C> for B {
    const EQUAL: bool = C::EQUAL;
}

/// Represent a searchable `Hlist`
///
/// Means we can check for target type `T` in `Self` using comparator `C`.
pub unsafe trait Searchable<T, C> {
    /// Can compare `Head` of `Hlist` with target type `T`
    type Head: CompareWith<T, C>;
    /// Can recursively search the `Tail` of `Hlist`
    type Tail: Searchable<T, C>;
    /// This should be set to true for [End]
    const END: bool = false;
}

unsafe impl<T, C, Head, Tail> Searchable<T, C> for Cons<Head, Tail>
where
    Head: CompareWith<T, C>,
    Tail: Searchable<T, C>,
{
    type Head = Head;
    type Tail = Tail;
}

unsafe impl<T, C> Searchable<T, C> for End {
    type Head = End;
    type Tail = End;
    const END: bool = true;
}

#[track_caller]
pub const fn expect_some<T: Copy>(o: Option<T>, err_msg: &'static str) -> T {
    match o {
        Some(t) => t,
        None => panic!("{}", err_msg),
    }
}

pub const fn get_cons_offset<T, C, L: Searchable<T, C>>() -> Option<isize> {
    get_cons_offset_helper::<T, C, L>(0)
}

// recursively look for type T in an Hlist
// offset should be initially zero
// offset is in bytes until we find G, then convert to offset for G
const fn get_cons_offset_helper<T, C, L: Searchable<T, C>>(offset: isize) -> Option<isize> {
    if L::END {
        None
    }
    else if <L::Head as CompareWith<T, C>>::EQUAL {
        // offset is in bytes, so when we found our type, divide by type size
        // for the proper offset
        // assert!(offset + std::mem::size_of::<G>() as isize <= isize::MAX);
        let head_size = std::mem::size_of::<T>();
        if head_size == 0 {
            Some(offset)
        }
        else {
            Some(offset / std::mem::size_of::<T>() as isize)
        }
    }
    else {
        let head_size = std::mem::size_of::<L::Head>();
        let tail_align = std::mem::align_of::<L::Tail>();
        let align_diff = head_size % tail_align;
        let padding = if align_diff != 0 {
            tail_align - align_diff
        }
        else {
            0
        };
        get_cons_offset_helper::<T, C, L::Tail>(offset + head_size as isize + padding as isize)
    }
}