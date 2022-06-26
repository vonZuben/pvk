use crate::{ Cons, End };

use super::{ FuncMut, FuncMutOut, Gat };

pub type FoldRefOut<'a, S, I, F> = <<S as FoldRef<I, F>>::OutputTypeConstructor as Gat<'a>>::Gat;

pub trait FoldRef<I, F> {
    type OutputTypeConstructor: ?Sized + for<'a> Gat<'a>;
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