use std::{marker::PhantomData, ptr::NonNull};

#[cfg(test)]
pub mod tests;

pub mod sealed {
    pub trait Sealed {}
}

use sealed::Sealed;

#[derive(Default)]
pub struct Link<Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    next: Option<NonNull<T>>,
    prev: Option<NonNull<T>>,
    owner: Option<NonNull<List<Tag, T>>>,
    _marker: PhantomData<Tag>,
}

pub trait HasLink<Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    fn link(&self) -> &Link<Tag, T>;
    fn link_mut(&mut self) -> &mut Link<Tag, T>;
}

#[derive(Debug)]
pub struct List<Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
    _marker: PhantomData<Tag>,
}

impl<Tag, T> List<Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, mut item: NonNull<T>) {
        unsafe {
            let link = item.as_mut().link_mut();
            link.next = None;
            link.prev = self.tail;
            link.owner = Some(NonNull::new_unchecked(self as *mut _));

            if let Some(mut tail) = self.tail {
                tail.as_mut().link_mut().next = Some(item);
            } else {
                self.head = Some(item);
            }

            self.tail = Some(item);
        }
    }

    pub fn pop(&mut self) -> Option<NonNull<T>> {
        self.tail.map(|tail| unsafe {
            let prev = tail.as_ref().link().prev;

            if let Some(mut prev) = prev {
                prev.as_mut().link_mut().next = None;
                self.tail = Some(prev);
            } else {
                self.head = None;
                self.tail = None;
            }

            tail
        })
    }

    pub fn insert_before(&mut self, mut target: NonNull<T>, mut new_item: NonNull<T>) {
        unsafe {
            let prev = target.as_ref().link().prev;

            let new_link = new_item.as_mut().link_mut();
            new_link.next = Some(target);
            new_link.prev = prev;
            new_link.owner = Some(NonNull::new_unchecked(self as *mut _));

            target.as_mut().link_mut().prev = Some(new_item);

            if let Some(mut prev) = prev {
                prev.as_mut().link_mut().next = Some(new_item);
            } else {
                self.head = Some(new_item);
            }
        }
    }

    pub fn insert_after(&mut self, mut target: NonNull<T>, mut new_item: NonNull<T>) {
        unsafe {
            let next = target.as_ref().link().next;

            let new_link = new_item.as_mut().link_mut();
            new_link.prev = Some(target);
            new_link.next = next;
            new_link.owner = Some(NonNull::new_unchecked(self as *mut _));

            target.as_mut().link_mut().next = Some(new_item);

            if let Some(mut next) = next {
                next.as_mut().link_mut().prev = Some(new_item);
            } else {
                self.tail = Some(new_item);
            }
        }
    }

    pub fn remove(&mut self, item: NonNull<T>) {
        unsafe {
            let prev = item.as_ref().link().prev;
            let next = item.as_ref().link().next;

            if let Some(mut prev) = prev {
                prev.as_mut().link_mut().next = next;
            } else {
                self.head = next;
            }

            if let Some(mut next) = next {
                next.as_mut().link_mut().prev = prev;
            } else {
                self.tail = prev;
            }
        }
    }

    pub fn iter<'list>(&self) -> Iter<'list, Tag, T> {
        Iter {
            next: self.head,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut<'list>(&self) -> IterMut<'list, Tag, T> {
        IterMut {
            next: self.head,
            _marker: PhantomData,
        }
    }

    pub fn drain<'list>(&'list mut self) -> DrainIter<'list, Tag, T> {
        DrainIter { list: self }
    }
}

pub struct Iter<'list, Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    next: Option<NonNull<T>>,
    _marker: PhantomData<&'list (Tag, T)>,
}

pub struct IterMut<'list, Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    next: Option<NonNull<T>>,
    _marker: PhantomData<&'list mut (Tag, T)>,
}

pub struct DrainIter<'list, Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    list: &'list mut List<Tag, T>,
}

impl<'list, Tag, T> Iterator for Iter<'list, Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    type Item = &'list T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|current| unsafe {
            let current_ref = current.as_ref();
            self.next = current_ref.link().next;
            current_ref
        })
    }
}

impl<'list, Tag, T> Iterator for IterMut<'list, Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    type Item = &'list mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|mut current| unsafe {
            let current_mut = current.as_mut();
            self.next = current_mut.link().next;
            current_mut
        })
    }
}

impl<'list, Tag, T> Iterator for DrainIter<'list, Tag, T>
where
    Tag: Sealed,
    T: HasLink<Tag, T>,
{
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.head.map(|head| {
            self.list.remove(head);
            head
        })
    }
}
