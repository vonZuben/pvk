//! Higher-order functionality for hlist

mod map;
mod for_each;
mod for_each_mut;

pub mod higher_order_prelude {
    use super::*;

    pub use map::Map;
    pub use for_each::ForEach;
    pub use for_each_mut::ForEachMut;

    pub use super::FuncMut;
    pub use super::Gat;
}

pub trait FuncMut<Input> {
    type Output;
    fn call_mut(&mut self, i: Input) -> Self::Output;
}

pub trait Gat<'a> {
    type Gat;
}
