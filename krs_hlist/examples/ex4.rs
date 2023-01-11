
use krs_hlist::{Comparator, hlist};

unsafe trait Id {
    const ID: usize;
}

struct Comp;

unsafe impl<A: Id, B: Id> Comparator<A, B> for Comp {
    const EQUAL: bool = A::ID == B::ID;
}

#[derive(Debug)]
struct A(i8);

#[derive(Debug)]
struct B(u32);

#[derive(Debug)]
struct C(f32);

#[derive(Debug)]
struct D(i16);

macro_rules! unsafe_impl_id {
    ( $( $name:ident : $id:literal ),* $(,)? ) => {
        $(
            unsafe impl Id for $name {
                const ID: usize = $id;
            }
        )*
    };
}

unsafe_impl_id!{
    A: 1,
    B: 2,
    C: 3,
    D: 4,
}

trait Contains<T> : krs_hlist::Contains<T, Comp> {}
impl<T, L> Contains<T> for L where L: krs_hlist::Contains<T, Comp> {}

fn tst(list: impl Contains<C>) {
    let c: &C = list.get();

    // TODO: see below note regarding "associated_const_equality"
    // let c: &B = list.get();

    println!("{c:?}");
}

fn main() {
    // TODO: At this time, I like to think that the Contains trait represents that a collection contains a type
    // but this is incorrect since as seen below, 'list' does not contain B which should be required for 'tst'.
    //
    // Hopefully the feature "associated_const_equality" becomes stable. Afterward, the Contains trait can be used
    // as a question regarding if a collection contains a type, and another trait (which I plan to call Get), will
    // be implemented for types that *must* contain a specific type (e.g. Get<T> for L where L: Contains<T, OFFSET.is_some()>)
    let list = hlist!(A(1), A(2), B(3), D(4), C(5.0));
    tst(list);
}