use std::ops::Add;

mod const_utils;

pub use const_utils::Comparator;

#[macro_export]
macro_rules! hlist {
    ( $( $val:expr ),* $(,)? ) => {{
        let list = $crate::End;
        $(
            let list = list + $crate::Cons::new($val);
        )*
        list
    }};
}

/// represent hlisty things
pub trait Hlist {
    type Head;
    type Tail: Hlist;
    const LEN: usize;
}

/// The main building block of hlist
/// an hlist is a chain of this type
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Cons<H, T> {
    head: H,
    tail: T,
}

impl<H> Cons<H, End> {
    pub fn new(head: H) -> Self {
        Cons { head, tail: End }
    }
}

impl<H, T: Hlist> Hlist for Cons<H, T> {
    type Head = H;
    type Tail = T;
    const LEN: usize = T::LEN + 1;
}

/// Mark the end of an hlist
#[derive(Debug, Clone, Copy)]
pub struct End;

impl Hlist for End {
    type Head = End;
    type Tail = End;
    const LEN: usize = 0;
}

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

impl<RHS: Hlist> Add<RHS> for End {
    type Output = RHS;
    fn add(self, rhs: RHS) -> Self::Output {
        rhs
    }
}

pub trait Contains<T, C> {
    const OFFSET: isize;
    // const HAS: bool = Self::OFFSET.is_some();  // for use with "associated_const_equality", and OFFSET should be Option<isize>
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

    // THIS IS A COPY OF THE NOTE IN THE EXAMPLE ex1.rs
    // TODO: At this time, I like to think that the Contains trait represents that a collection contains a type
    // but this is incorrect since as seen below, 'list' does not contain B which should be required for 'tst'.
    //
    // Hopefully the feature "associated_const_equality" becomes stable. Afterward, the Contains trait can be used 
    // as a question regarding if a collection contains a type, and another trait (which I plan to call Get), will
    // be implemented for types that *must* contain a specific type (e.g. Get<T> for L where L: Contains<T, OFFSET.is_some()>)
impl<T, C, L: const_utils::Searchable<T, C>> Contains<T, C> for L {
    const OFFSET: isize = const_utils::expect_some(const_utils::get_cons_offset::<T, C, L>(), "error: type T is not in hlist");
    fn get(&self) -> &T {
        let t_ptr: *const T = (self as *const Self).cast();
        unsafe { &*t_ptr.offset(<Self as Contains<T, C>>::OFFSET) }
    }
    fn get_mut(&mut self) -> &mut T {
        let t_ptr: *mut T = (self as *mut Self).cast();
        unsafe { &mut *t_ptr.offset(<Self as Contains<T, C>>::OFFSET) }
    }
}

/// Get T using C as a Comparator
/// if "associated_const_equality" becomes stable
// trait Get<T, C> : Contains<T, C, HAS==true> {
//     fn get(&self) -> &T;
//     fn get_mut(&self) -> &mut T;
// }

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