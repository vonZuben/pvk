//! Run function for each element of hist, by `&` of each element

use crate::{ Cons, End };

use super::{ FuncMut, FuncMutOut, Gat };

/// Convenience alias to get the output from [for_each](ForEach::for_each)
pub type ForEachOut<'a, S, F> = <<S as ForEach<F>>::OutputTypeConstructor as Gat<'a>>::Gat;

/// "For Each" operation of items in an `Hlist` by `&`
///
/// `F` should be an implementor of [FuncMut], which should generically works for
/// all types in the `Hlist` by `&`
pub trait ForEach<F> {
    /// Type constructor to generate type with generic lifetime
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
    /// call a generic function for each element of an hlist by `&`
    /// Similar to [Iterator::for_each], but for `Hlist` and can have
    /// an output value for each element
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
