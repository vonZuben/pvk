
use krs_hlist::{Comparator, Cons, End};

unsafe trait Id {
    const ID: usize;
}

struct Comp;

unsafe impl<A: Id, B: Id> Comparator<A, B> for Comp {
    const EQUAL: bool = A::ID == B::ID;
}

#[derive(Debug)]
struct A;

#[derive(Debug)]
struct B;

#[derive(Debug)]
struct C;

#[derive(Debug)]
struct D;

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

trait Contains<T> : krs_hlist::Get<T, Comp> {}
impl<T, L> Contains<T> for L where L: krs_hlist::Get<T, Comp> {}

trait Ver10 : Contains<A> + Contains<B> {}
impl<V> Ver10 for V where V: Contains<A> + Contains<B> {}

fn tst(list: impl Ver10) {
    let c: &A = list.get();

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
    let list = Cons::new(A, Cons::new(B, End));
    tst(list);
}