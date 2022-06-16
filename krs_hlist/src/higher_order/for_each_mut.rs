use crate::{ Cons, End };

use super::{ FuncMut, Gat };

pub trait ForEachMut<F> {
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
    fn for_each_mut<'a>(&'a mut self, f: F) -> <Self::OutputTypeConstructor as Gat<'a>>::Gat;
}

impl<F, Head, Tail> ForEachMut<F> for Cons<Head, Tail>
where
    F: for<'a> FuncMut<&'a mut Head>,
    Tail: ForEachMut<F>,
{
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = Cons<<F as FuncMut<&'a mut Head>>::Output, <Tail::OutputTypeConstructor as Gat<'a>>::Gat> >;
    fn for_each_mut(&mut self, mut f: F) -> <Self::OutputTypeConstructor as Gat>::Gat {
        Cons{ head: f.call_mut(&mut self.head), tail: self.tail.for_each_mut(f) }
    }
}

impl<F> ForEachMut<F> for End {
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = End>;
    fn for_each_mut(&mut self, _f: F) -> <Self::OutputTypeConstructor as Gat>::Gat {
        End
    }
}

