//! Fold each element of the hlist, by `&` of each element

use crate::{ Cons, End };

use super::{ FuncMut, FuncMutOut, Gat };

/// Convenience alias to get the output from [fold_ref](FoldRef::fold_ref)
pub type FoldRefOut<'a, S, I, F> = <<S as FoldRef<I, F>>::OutputTypeConstructor as Gat<'a>>::Gat;

/// Fold operation of items in an `Hlist` by `&`
///
/// `F` should be an implementor of [FuncMut], which should generically works for
/// all types in the `Hlist`
///
/// *Note*: The type of the accumulator does not need to be fixed, anc can be a different type
/// for each different input type
pub trait FoldRef<I, F> {
    /// Type constructor to generate type with generic lifetime
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
    /// Folds every element into an accumulator by applying an operation, returning the final result.
    /// Similar to [Iterator::fold], but for `Hlist`
    fn fold_ref<'a>(&'a self, init: I, f: F) -> FoldRefOut<'a, Self, I, F>;
}

impl<I, F, Head, Tail> FoldRef<I, F> for Cons<Head, Tail>
where
    F: for<'a> FuncMut<(I, &'a Head)>,
    Tail: for<'a> FoldRef<FuncMutOut<F, (I, &'a Head)>, F>,
{
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = FoldRefOut<'a, Tail, FuncMutOut<F, (I, &'a Head)>, F>>;
    fn fold_ref<'a>(&'a self, init: I, mut f: F) -> FoldRefOut<'a, Self, I, F> {
        let next = f.call_mut((init, &self.head));
        self.tail.fold_ref(next, f)
    }
}

impl<I, F> FoldRef<I, F> for End {
    type OutputTypeConstructor = dyn for<'a> Gat<'a, Gat = I>;
    fn fold_ref<'a>(&'a self, init: I, _f: F) -> FoldRefOut<'a, Self, I, F> {
        init
    }
}