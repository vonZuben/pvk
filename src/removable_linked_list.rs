
use std::mem;
use std::ptr;
use std::marker::PhantomData;

use std::iter::{FromIterator, IntoIterator};

type BoxRListNode<T> = Box<RListNode<T>>;

// RList can be safely sent between threads
// if any iteration is happening on the source thread
// then the borow checker prevent moving to new thread
// this type is not SYNC however though
// because it creates Drones which are not thread safe
unsafe impl<T> std::marker::Send for RList<T> where T: std::fmt::Debug {}

#[derive(Debug)]
pub struct RList<T> where T: std::fmt::Debug {
    head: Option<BoxRListNode<T>>,
    tail: *mut RListNode<T>,
}

impl<T> FromIterator<T> for RList<T> where T: std::fmt::Debug {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut ll = RList::new();

        for i in iter {
            ll.add(i);
        }

        ll
    }
}

impl<'a, T> IntoIterator for &'a mut RList<T> where T: std::fmt::Debug {
    type Item = RListDrone<'a, T>;
    type IntoIter = RListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> RList<T> where T: std::fmt::Debug{

    pub fn new() -> Self {
        RList {
            head: None,
            tail: ptr::null_mut(),
        }
    }

    pub fn add(&mut self, val: T) {
        match &self.head {
            None => {

                let node = RListNode::new(val);

                let mut bn = Box::new(node);
                self.tail = bn.as_mut() as *mut RListNode<T>;
                self.head = Some(bn.into());

            }
            Some(_head) => {

                let mut node = RListNode::new(val);
                node.prev = self.tail;

                let mut bn = Box::new(node);
                let new_tail = bn.as_mut() as *mut RListNode<T>;
                unsafe { (*self.tail).next = Some(bn) }

                self.tail = new_tail;

            }
        }
    }

    pub fn iter<'a>(&'a mut self) -> RListIter<'a, T> {
        match self.head {
            None => {
                RListIter {
                    cur: ptr::null_mut(),
                    ll: self,
                }
            }
            Some(ref mut head) => {
                RListIter {
                    cur: head.as_mut() as *mut RListNode<T>,
                    ll: self,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RListIter<'a, T> where T: std::fmt::Debug{
    cur: *mut RListNode<T>,
    ll: &'a mut RList<T>,
}

impl<'a, T> RListIter<'a, T> where T: std::fmt::Debug{

    fn make_drone(&mut self) -> RListDrone<'a, T> {
        RListDrone {
            node: self.cur,
            ll: self.ll as *mut RList<T>,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct RListDrone<'a, T> where T: std::fmt::Debug{
    node: *mut RListNode<T>,
    ll: *mut RList<T>,
    phantom: PhantomData<&'a mut RList<T>>,
}

impl<'a, T> RListDrone<'a, T> where T: std::fmt::Debug{

    pub fn data(&self) -> &'a T {
        unsafe { &(*self.node).data }
    }

    pub fn remove(self) -> T {

        let node = unsafe { self.node.as_mut().expect("empty node in drone") };

        unsafe {
            match node.prev.as_mut() {
                None => {
                    let ll = self.ll.as_mut().expect("empty ll in drone");
                    let mut next = mem::replace(&mut node.next, None);
                    match next {
                        Some(ref mut next) => next.prev = ptr::null_mut(),
                        None => {},
                    }
                    let node = mem::replace(&mut ll.head, next);
                    node.expect("empty head").data
                }
                Some(prev) => {
                    let mut next = mem::replace(&mut node.next, None);
                    match next {
                        Some(ref mut next) => next.prev = node.prev,
                        None => {},
                    }
                    let node = mem::replace(&mut prev.next, next);
                    node.expect("unexpexted empty node").data
                }
            }
        }

    }
}

impl<'a, T> Iterator for RListIter<'a, T>where T: std::fmt::Debug{
    type Item = RListDrone<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {

        if self.cur.is_null() {
            None
        }
        else {
            let ll_node = self.make_drone();

            self.cur = unsafe { match (*self.cur).next {
                        Some(ref mut next) => next.as_mut() as *mut RListNode<T>,
                        None => ptr::null_mut(),
                    }
                };

            Some(ll_node)
        }

    }
}

#[derive(Debug)]
struct RListNode<T> where T: std::fmt::Debug {
    prev: *mut RListNode<T>,
    next: Option<BoxRListNode<T>>,
    data: T,
}

impl<T> RListNode<T> where T: std::fmt::Debug {

    fn new(val: T) -> Self {
        RListNode {
            prev: ptr::null_mut(),
            next: None,
            data: val,
        }
    }
}
