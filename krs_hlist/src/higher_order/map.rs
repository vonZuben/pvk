//! Run function for each element of hist, by moving each element

use crate::{ Cons, End };

use super::FuncMut;

pub type MapOut<S, F> = <S as Map<F>>::Output;

pub trait Map<F> {
    type Output;
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
