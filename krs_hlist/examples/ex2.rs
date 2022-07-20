
use krs_hlist::{ Cons, End, higher_order::prelude::* };

struct Add(i32);

impl<I> FuncMut<&mut I> for Add
where
    I: std::ops::AddAssign<i32>,
{
    type Output = ();
    fn call_mut(&mut self, i: &mut I) {
        *i += self.0;
    }
}

fn main() {
    let mut list = Cons::new(1, Cons::new(2, Cons::new(3, End)));

    list.for_each_mut(Add(10));

    println!("{list:?}");
}
