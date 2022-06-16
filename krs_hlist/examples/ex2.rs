
use krs_hlist::{ hlist, higher_order_prelude::* };

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
    let mut list = hlist!(1, 2, 3);

    list.for_each_mut(Add(10));

    println!("{list:?}");
}
