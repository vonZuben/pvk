//! Higher-order functionality for hlist

mod map;
mod for_each;
mod for_each_mut;

pub mod higher_order_prelude {
    use super::*;

    // traits for higher order functions
    pub use map::Map;
    pub use for_each::ForEach;
    pub use for_each_mut::ForEachMut;

    // convenience types for higher order function outputs
    pub use map::MapOut;
    pub use for_each::ForEachOut;
    pub use for_each_mut::ForEachMutOut;

    // needed for users to implement their own functionality for higher order functions
    pub use super::FuncMut;
}

pub trait FuncMut<Input> {
    type Output;
    fn call_mut(&mut self, i: Input) -> Self::Output;
}

pub type FuncMutOut<F, I> = <F as FuncMut<I>>::Output;

pub trait Gat<'a> {
    type Gat;
}
