//! Run function for each element of hist, by `&mut` of each element

use crate::{ Cons, End };

use super::{ FuncMut, FuncMutOut, Gat };

/// Convenience alias to get the output from [for_each_mut](ForEachMut::for_each_mut)
pub type ForEachMutOut<'a, S, F> = <<S as ForEachMut<F>>::OutputTypeConstructor as Gat<'a>>::Gat;

/// "For Each" operation of items in an `Hlist` by `&mut`
///
/// `F` should be an implementor of [FuncMut], which should generically works for
/// all types in the `Hlist` by `&mut`
pub trait ForEachMut<F> {
    /// Type constructor to generate type with generic lifetime
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
    /// call a generic function for each element of an hlist by `&mut`
    /// Similar to [Iterator::for_each], but for `Hlist` and can have
    /// an output value for each element
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

