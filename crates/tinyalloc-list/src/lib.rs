use getset::{Getters, Setters};
use std::ptr::NonNull;

#[cfg(test)]
pub mod tests;

#[derive(Debug, Default, Getters, Setters)]
pub struct Link<T>
where
    T: HasLink<T>,
{
    #[getset(get = "pub", set = "pub")]
    next: Option<NonNull<T>>,
    #[getset(get = "pub", set = "pub")]
    prev: Option<NonNull<T>>,
    #[getset(get = "pub", set = "pub")]
    owner: Option<NonNull<List<T>>>,
}

impl<T> Link<T>
where
    T: HasLink<T>,
{
    pub fn clear(&mut self) {
        self.set_next(None);
        self.set_prev(None);
        self.set_owner(None);
    }

    pub fn set_list_owner(&mut self, owner: &List<T>) {
        self.set_owner(Some(unsafe {
            NonNull::new_unchecked(owner as *const _ as *mut _)
        }));
    }

    pub fn is_owned_by(&self, owner: &List<T>) -> bool {
        if let Some(link_owner) = self.owner() {
            std::ptr::eq(link_owner.as_ptr(), owner as *const _)
        } else {
            false
        }
    }
}

pub trait HasLink<T>
where
    T: HasLink<T>,
{
    fn link(&self) -> &Link<T>;
    fn link_mut(&mut self) -> &mut Link<T>;
}

#[derive(Debug)]
pub struct List<T>
where
    T: HasLink<T>,
{
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
}

impl<T> List<T>
where
    T: HasLink<T>,
{
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn contains(&self, item: NonNull<T>) -> bool {
        unsafe { item.as_ref().link().is_owned_by(self) }
    }

    pub fn push(&mut self, mut item: NonNull<T>) {
        unsafe {
            let link = item.as_mut().link_mut();
            link.clear();
            link.set_prev(self.tail);
            link.set_list_owner(self);

            if let Some(mut tail) = self.tail {
                tail.as_mut().link_mut().set_next(Some(item));
            } else {
                self.head = Some(item);
            }

            self.tail = Some(item);
        }
    }

    pub fn push_front(&mut self, mut item: NonNull<T>) {
        unsafe {
            let link = item.as_mut().link_mut();
            link.clear();
            link.set_next(self.head);
            link.set_list_owner(self);

            if let Some(mut head) = self.head {
                head.as_mut().link_mut().set_prev(Some(item));
            } else {
                self.tail = Some(item);
            }

            self.head = Some(item);
        }
    }

    pub fn pop_front(&mut self) -> Option<NonNull<T>> {
        self.head.map(|mut head| unsafe {
            let next = *head.as_ref().link().next();

            if let Some(mut next) = next {
                next.as_mut().link_mut().set_prev(None);
                self.head = Some(next);
            } else {
                self.head = None;
                self.tail = None;
            }

            head.as_mut().link_mut().clear();

            head
        })
    }

    pub fn pop(&mut self) -> Option<NonNull<T>> {
        self.tail.map(|mut tail| unsafe {
            let prev = *tail.as_ref().link().prev();

            if let Some(mut prev) = prev {
                prev.as_mut().link_mut().set_next(None);
                self.tail = Some(prev);
            } else {
                self.head = None;
                self.tail = None;
            }

            tail.as_mut().link_mut().clear();

            tail
        })
    }

    pub fn insert_before(&mut self, mut target: NonNull<T>, mut new_item: NonNull<T>) {
        unsafe {
            let prev = *target.as_ref().link().prev();

            let new_link = new_item.as_mut().link_mut();
            new_link.clear();
            new_link.set_next(Some(target));
            new_link.set_prev(prev);
            new_link.set_list_owner(self);

            target.as_mut().link_mut().set_prev(Some(new_item));

            if let Some(mut prev) = prev {
                prev.as_mut().link_mut().set_next(Some(new_item));
            } else {
                self.head = Some(new_item);
            }
        }
    }

    pub fn insert_after(&mut self, mut target: NonNull<T>, mut new_item: NonNull<T>) {
        unsafe {
            let next = *target.as_ref().link().next();

            let new_link = new_item.as_mut().link_mut();
            new_link.clear();
            new_link.set_prev(Some(target));
            new_link.set_next(next);
            new_link.set_list_owner(self);

            target.as_mut().link_mut().set_next(Some(new_item));

            if let Some(mut next) = next {
                next.as_mut().link_mut().set_prev(Some(new_item));
            } else {
                self.tail = Some(new_item);
            }
        }
    }

    pub fn remove_unchecked(&mut self, mut item: NonNull<T>) {
        unsafe {
            let link = item.as_mut().link_mut();
            let prev = *link.prev();
            let next = *link.next();

            if let Some(mut prev) = prev {
                prev.as_mut().link_mut().set_next(next);
            } else {
                self.head = next;
            }

            if let Some(mut next) = next {
                next.as_mut().link_mut().set_prev(prev);
            } else {
                self.tail = prev;
            }

            link.clear();
        }
    }

    pub fn remove(&mut self, item: NonNull<T>) -> bool {
        if self.contains(item) {
            self.remove_unchecked(item);
            true
        } else {
            false
        }
    }

    pub fn is_linked(&self, item: NonNull<T>) -> bool {
        unsafe { item.as_ref().link().owner().is_some() }
    }

    pub fn iter<'list>(&'list self) -> Iter<'list, T> {
        Iter {
            list: self,
            next: self.head,
        }
    }

    pub fn iter_mut<'list>(&'list self) -> IterMut<'list, T> {
        IterMut {
            list: self,
            next: self.head,
        }
    }

    pub fn drain<'list>(&'list mut self) -> DrainIter<'list, T> {
        DrainIter { list: self }
    }
}

pub struct Iter<'list, T>
where
    T: HasLink<T>,
{
    list: &'list List<T>,
    next: Option<NonNull<T>>,
}

pub struct IterMut<'list, T>
where
    T: HasLink<T>,
{
    list: &'list List<T>,
    next: Option<NonNull<T>>,
}

pub struct DrainIter<'list, T>
where
    T: HasLink<T>,
{
    list: &'list mut List<T>,
}

impl<'list, T> Iterator for Iter<'list, T>
where
    T: HasLink<T>,
{
    type Item = &'list T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.next {
            unsafe {
                let current_ref = current.as_ref();
                self.next = *current_ref.link().next();

                if current_ref.link().is_owned_by(self.list) {
                    return Some(current_ref);
                }
            }
        }
        None
    }
}

impl<'list, T> Iterator for IterMut<'list, T>
where
    T: HasLink<T>,
{
    type Item = &'list mut T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut current) = self.next {
            unsafe {
                let current_mut = current.as_mut();
                self.next = *current_mut.link().next();

                if current_mut.link().is_owned_by(self.list) {
                    return Some(current_mut);
                }
            }
        }
        None
    }
}

impl<'list, T> Iterator for DrainIter<'list, T>
where
    T: HasLink<T>,
{
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}
