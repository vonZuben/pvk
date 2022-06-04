use super::{Cons, End};

pub unsafe trait Comparator<A: ?Sized, B: ?Sized> {
    const EQUAL: bool;
}

unsafe impl<T: ?Sized, C> Comparator<T, End> for C {
    const EQUAL: bool = false;
}

pub trait CompareWith<T, C> {
    const EQUAL: bool;
}

impl<A, B, C: Comparator<A, B>> CompareWith<A, C> for B {
    const EQUAL: bool = C::EQUAL;
}


pub unsafe trait Searchable<T, C> {
    type Head: CompareWith<T, C>;
    type Tail: Searchable<T, C>;
    const END: bool = false;
}

unsafe impl<T, C, Head, Tail> Searchable<T, C> for Cons<Head, Tail> 
where
    Head: CompareWith<T, C>,
    Tail: Searchable<T, C>,
{
    type Head = Head;
    type Tail = Tail;
}

// for being able to check End
/////////////////////
unsafe impl<T, C> Searchable<T, C> for End {
    type Head = End;
    type Tail = End;
    const END: bool = true;
}

/////////////////////

#[track_caller]
pub const fn expect_some<T: Copy>(o: Option<T>, err_msg: &'static str) -> T {
    match o {
        Some(t) => t,
        None => panic!("{}", err_msg),
    }
}

pub const fn get_cons_offset<T, C, L: Searchable<T, C>>() -> Option<isize> {
    get_cons_offset_helper::<T, C, L>(0)
}

// recursively look for type T in an Hlist
// offset should be initially zero
// offset is in bytes until we find G, then convert to offset for G
const fn get_cons_offset_helper<T, C, L: Searchable<T, C>>(offset: isize) -> Option<isize> {
    if L::END {
        None
    }
    else if <L::Head as CompareWith<T, C>>::EQUAL {
        // offset is in bytes, so when we found our type, divide by type size
        // for the proper offset
        // assert!(offset + std::mem::size_of::<G>() as isize <= isize::MAX);
        let head_size = std::mem::size_of::<T>();
        if head_size == 0 {
            Some(offset)
        }
        else {
            Some(offset / std::mem::size_of::<T>() as isize)     
        }
    }
    else {
        let head_size = std::mem::size_of::<L::Head>();
        let tail_align = std::mem::align_of::<L::Tail>();
        let align_diff = head_size % tail_align;
        let padding = if align_diff != 0 {
            tail_align - align_diff
        }
        else {
            0
        };
        get_cons_offset_helper::<T, C, L::Tail>(offset + head_size as isize + padding as isize)
    }
}