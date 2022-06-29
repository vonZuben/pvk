//! Higher-order functionality for hlist
//!
//! Top level module containing all the different higher-order functionality traits

pub mod map;
pub mod for_each;
pub mod for_each_mut;
pub mod fold_ref;

/// Prelude for convenient inclusion of all higher-order functionality traits
pub mod prelude {
    use super::*;

    // traits for map like higher order functions
    pub use map::Map;
    pub use for_each::ForEach;
    pub use for_each_mut::ForEachMut;

    // convenience types for map like higher order function outputs
    pub use map::MapOut;
    pub use for_each::ForEachOut;
    pub use for_each_mut::ForEachMutOut;

    // trait for fold like higher order functions
    pub use fold_ref::FoldRef;

    // convenience types for map like higher order function outputs
    pub use fold_ref::FoldRefOut;

    // needed for users to implement their own functionality for higher order functions
    pub use super::FuncMut;
}

/// For custom higher order functionality
///
/// The normal rust 'Fn' traits can't be generic over input types. This trait allows us to create types
/// that represent functions that are generic over input types.
pub trait FuncMut<Input> {
    /// return type for calling the function with the given `Input` type
    type Output;
    /// Call the function
    ///
    /// Takes &mut self so that it can be used like a closure with state which can be mutated if necessary.
    fn call_mut(&mut self, i: Input) -> Self::Output;
}

/// Convenience type for output of a `FuncMut` type `F` when given input `I`
pub type FuncMutOut<F, I> = <F as FuncMut<I>>::Output;

#[doc(hidden)]
// This is a hack in order to have generic associated lifetimes. Based on https://gist.github.com/jix/42d0e4a36ace4c618a59f0ba03be5bf5
// see use in the different higher order function traits
pub trait Gat<'a> {
    type Gat;
}
