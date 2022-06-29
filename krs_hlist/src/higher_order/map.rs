//! Run function for each element of hist, by moving each element

use crate::{ Cons, End };

use super::FuncMut;

/// Convenience alias to get the output from [map](Map::map)
pub type MapOut<S, F> = <S as Map<F>>::Output;

/// "Map" operation of items in an `Hlist`
///
/// `F` should be an implementor of [FuncMut], which should generically works for
/// all types in the `Hlist`
///
/// maps an `Hlist[[A, B, C]]` to `Hlist[[D, E, D]]`.
pub trait Map<F> {
    /// Output hlist
    type Output;
    /// call a generic function for each element of an hlist
    /// similar to [Iterator::map], but not lazy
    fn map(self, f: F) -> Self::Output;
}

impl<F, Head, Tail> Map<F> for Cons<Head, Tail>
where
    F: FuncMut<Head>,
    Tail: Map<F>,
{
    type Output = Cons<F::Output, Tail::Output>;
    fn map(self, mut f: F) -> Self::Output {
        Cons{ head: f.call_mut(self.head), tail: self.tail.map(f) }
    }
}

impl<F> Map<F> for End {
    type Output = End;
    fn map(self, _f: F) -> Self::Output {
        End
    }
}
