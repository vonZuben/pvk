use crate::{ Cons, End };

use super::{ FuncMut, FuncMutOut, Gat };

pub type ForEachMutOut<'a, S, F> = <<S as ForEachMut<F>>::OutputTypeConstructor as Gat<'a>>::Gat;

pub trait ForEachMut<F> {
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
    fn for_each_mut<'a>(&'a mut self, f: F) -> ForEachMutOut<'a, Self, F>;
}

impl<F, Head, Tail> ForEachMut<F> for Cons<Head, Tail>
where
    F: for<'a> FuncMut<&'a mut Head>,
    Tail: ForEachMut<F>,
{
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = Cons<FuncMutOut<F, &'a mut Head>, ForEachMutOut<'a, Tail, F>> >;
    fn for_each_mut(&mut self, mut f: F) -> ForEachMutOut<Self, F> {
        Cons{ head: f.call_mut(&mut self.head), tail: self.tail.for_each_mut(f) }
    }
}

impl<F> ForEachMut<F> for End {
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = End>;
    fn for_each_mut(&mut self, _f: F) -> ForEachMutOut<Self, F> {
        End
    }
}

