use std::ffi::c_char;

use crate::generated as vk;

mod private {
    pub struct CommandComparator;
}

/// const fn for comparing s strings
///
/// SAFETY : input must be valid c string pointers
const unsafe fn cmd_name_equal(mut c1_name: *const c_char, mut c2_name: *const c_char) -> bool {
    while *c1_name != '\0' as i8 && *c1_name == *c2_name {
        c1_name = c1_name.add(1);
        c2_name = c2_name.add(1);
    }
    *c1_name == *c2_name
}

unsafe impl<C1: vk::VulkanCommand, C2: vk::VulkanCommand> krs_hlist::Comparator<C1, C2> for private::CommandComparator {
    const EQUAL: bool = unsafe { cmd_name_equal(C1::VK_NAME, C2::VK_NAME) };
}

pub trait GetCommand<C> : krs_hlist::Get<C, private::CommandComparator> {}
impl<C, L> GetCommand<C> for L where L: krs_hlist::Get<C, private::CommandComparator> {}

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

// #[macro_export]
// macro_rules! hlist {
//         () => {
//             $crate::End
//         };
//         ( $last:expr $(,)? ) => {
//             // $crate::Hnode { ex: $last , tail: $crate::End }
//             $crate::Hnode::new($last)
//         };
//         ( $first:expr , $($rest:expr),* $(,)? ) => {
//             // $crate::Hnode { ex: $first , tail: ex!($($rest),*) }
//             $crate::Hnode::new($first) + hlist!($($rest),*)
//         };
//     }

// #[macro_export]
macro_rules! hlist_ty {
    () => {
        $crate::utils::End
    };
    ( $last:path $(,)? ) => {
        $crate::utils::Hnode<$last, $crate::utils::End>
    };
    ( $first:path , $($rest:path),* $(,)? ) => {
        $crate::utils::Hnode<$first , hlist_ty!($($rest),*)>
    };
}

// impl convenience functionality for VkResult
impl crate::generated::Result {
    pub fn is_err(&self) -> bool {
        self.0 < 0
    }
    pub fn is_success(&self) -> bool {
        self.0 >= 0
    }
}