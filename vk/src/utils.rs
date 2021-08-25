use std::marker::PhantomData;
use std::ops::Add;

pub trait Hlist {}

#[repr(C)]
pub struct Hnode<H, T> {
    pub head: H,
    pub tail: T,
}

impl<H> Hnode<H, End> {
    pub fn new(h: H) -> Self {
        Self { head: h, tail: End }
    }
}

impl<H, Tail> Hlist for Hnode<H, Tail> where Tail: Hlist {}

#[derive(Default)]
pub struct End;
impl Hlist for End {}

impl<RHS> Add<RHS> for End
where
    RHS: Hlist,
{
    type Output = RHS;
    fn add(self, rhs: RHS) -> RHS {
        rhs
    }
}

impl<E, T, RHS> Add<RHS> for Hnode<E, T>
where
    RHS: Hlist,
    T: Add<RHS>,
{
    type Output = Hnode<E, <T as Add<RHS>>::Output>;
    fn add(self, rhs: RHS) -> Self::Output {
        Hnode {
            head: self.head,
            tail: self.tail + rhs,
        }
    }
}

pub struct Here;
pub struct There<T>(PhantomData<T>);

pub trait Get<Type, Index> {
    fn get(&self) -> &Type;
}

impl<Type, Tail> Get<Type, Here> for Hnode<Type, Tail> {
    fn get(&self) -> &Type {
        &self.head
    }
}

impl<Head, Tail, FromTail, TailIndex> Get<FromTail, There<TailIndex>> for Hnode<Head, Tail>
where
    Tail: Get<FromTail, TailIndex>,
{
    fn get(&self) -> &FromTail {
        self.tail.get()
    }
}

#[macro_export]
macro_rules! hlist {
        () => {
            $crate::End
        };
        ( $last:expr $(,)? ) => {
            // $crate::Hnode { ex: $last , tail: $crate::End }
            $crate::Hnode::new($last)
        };
        ( $first:expr , $($rest:expr),* $(,)? ) => {
            // $crate::Hnode { ex: $first , tail: ex!($($rest),*) }
            $crate::Hnode::new($first) + hlist!($($rest),*)
        };
    }

#[macro_export]
macro_rules! hlist_ty {
        () => {
            $crate::End
        };
        ( $last:path $(,)? ) => {
            $crate::Hnode<$last, $crate::End>
        };
        ( $first:path , $($rest:path),* $(,)? ) => {
            $crate::Hnode<$first , hlist_ty!($($rest),*)>
        };
    }
