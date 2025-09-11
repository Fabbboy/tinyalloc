use std::{marker::PhantomData, ptr::NonNull};

#[cfg(test)]
pub mod tests;

pub trait Item {
    fn next(&self) -> Option<NonNull<Self>>;
    fn prev(&self) -> Option<NonNull<Self>>;
    fn set_next(&mut self, next: Option<NonNull<Self>>);
    fn set_prev(&mut self, prev: Option<NonNull<Self>>);
}

pub struct DrainIter<'list, T>
where
    T: Item,
{
    list: &'list mut List<T>,
}

pub struct Iter<'list, T>
where
    T: Item,
{
    next: Option<NonNull<T>>,
    _marker: PhantomData<&'list T>,
}

pub struct IterMut<'list, T>
where
    T: Item,
{
    next: Option<NonNull<T>>,
    _marker: PhantomData<&'list mut T>,
}

#[derive(Debug)]
pub struct List<T>
where
    T: Item,
{
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
}

impl<T> List<T>
where
    T: Item,
{
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn push(&mut self, mut item: NonNull<T>) {
        unsafe {
            item.as_mut().set_next(None);
            item.as_mut().set_prev(self.tail);

            if let Some(mut tail) = self.tail {
                tail.as_mut().set_next(Some(item));
            } else {
                self.head = Some(item);
            }

            self.tail = Some(item);
        }
    }

    pub fn pop(&mut self) -> Option<NonNull<T>> {
        self.tail.map(|tail| unsafe {
            let prev = tail.as_ref().prev();

            if let Some(mut prev) = prev {
                prev.as_mut().set_next(None);
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
            let prev = target.as_ref().prev();

            new_item.as_mut().set_next(Some(target));
            new_item.as_mut().set_prev(prev);

            target.as_mut().set_prev(Some(new_item));

            if let Some(mut prev) = prev {
                prev.as_mut().set_next(Some(new_item));
            } else {
                self.head = Some(new_item);
            }
        }
    }

    pub fn insert_after(&mut self, mut target: NonNull<T>, mut new_item: NonNull<T>) {
        unsafe {
            let next = target.as_ref().next();

            new_item.as_mut().set_prev(Some(target));
            new_item.as_mut().set_next(next);

            target.as_mut().set_next(Some(new_item));

            if let Some(mut next) = next {
                next.as_mut().set_prev(Some(new_item));
            } else {
                self.tail = Some(new_item);
            }
        }
    }

    pub fn remove(&mut self, item: NonNull<T>) {
        unsafe {
            let prev = item.as_ref().prev();
            let next = item.as_ref().next();

            if let Some(mut prev) = prev {
                prev.as_mut().set_next(next);
            } else {
                self.head = next;
            }

            if let Some(mut next) = next {
                next.as_mut().set_prev(prev);
            } else {
                self.tail = prev;
            }
        }
    }

    pub fn iter<'list>(&self) -> Iter<'list, T> {
        Iter {
            next: self.head,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut<'list>(&self) -> IterMut<'list, T> {
        IterMut {
            next: self.head,
            _marker: PhantomData,
        }
    }

    pub fn drain<'list>(&'list mut self) -> DrainIter<'list, T> {
        DrainIter { list: self }
    }
}

impl<'list, T> Iterator for Iter<'list, T>
where
    T: Item,
{
    type Item = &'list T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|current| unsafe {
            let current_ref = current.as_ref();
            self.next = current_ref.next();
            current_ref
        })
    }
}

impl<'list, T> Iterator for IterMut<'list, T>
where
    T: Item,
{
    type Item = &'list mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|mut current| unsafe {
            let current_mut = current.as_mut();
            self.next = current_mut.next();
            current_mut
        })
    }
}

impl<'list, T> Iterator for DrainIter<'list, T>
where
    T: Item,
{
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.head.map(|head| {
            self.list.remove(head);
            head
        })
    }
}
