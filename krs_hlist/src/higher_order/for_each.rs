use crate::{ Cons, End };

use super::{ FuncMut, FuncMutOut, Gat };

pub type ForEachOut<'a, S, F> = <<S as ForEach<F>>::OutputTypeConstructor as Gat<'a>>::Gat;

pub trait ForEach<F> {
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
    fn for_each<'a>(&'a self, f: F) -> ForEachOut<'a, Self, F>;
}

impl<F, Head, Tail> ForEach<F> for Cons<Head, Tail>
where
    F: for<'a> FuncMut<&'a Head>,
    Tail: ForEach<F>,
{
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = Cons<FuncMutOut<F, &'a Head>, ForEachOut<'a, Tail, F>> >;
    fn for_each(&self, mut f: F) -> ForEachOut<Self, F> {
        Cons{ head: f.call_mut(&self.head), tail: self.tail.for_each(f) }
    }
}

impl<F> ForEach<F> for End {
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = End>;
    fn for_each(&self, _f: F) -> ForEachOut<Self, F> {
        End
    }
}
