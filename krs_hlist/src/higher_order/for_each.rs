use crate::{ Cons, End };

use super::{ FuncMut, Gat };

pub trait ForEach<F> {
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
    fn for_each<'a>(&'a self, f: F) -> <Self::OutputTypeConstructor as Gat<'a>>::Gat;
}

impl<F, Head, Tail> ForEach<F> for Cons<Head, Tail>
where
    F: for<'a> FuncMut<&'a Head>,
    Tail: ForEach<F>,
{
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = Cons<<F as FuncMut<&'a Head>>::Output, <Tail::OutputTypeConstructor as Gat<'a>>::Gat> >;
    fn for_each(&self, mut f: F) -> <Self::OutputTypeConstructor as Gat>::Gat {
        Cons{ head: f.call_mut(&self.head), tail: self.tail.for_each(f) }
    }
}

impl<F> ForEach<F> for End {
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = End>;
    fn for_each(&self, _f: F) -> <Self::OutputTypeConstructor as Gat>::Gat {
        End
    }
}
