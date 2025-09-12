use std::ptr::NonNull;

use tinyalloc_list::Item;

pub struct Segment<'mapper> {
    next: Option<NonNull<Segment<'mapper>>>,
    prev: Option<NonNull<Segment<'mapper>>>,
    data: &'mapper [u8],
}

impl<'mapper> Item for Segment<'mapper> {
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn prev(&self) -> Option<NonNull<Segment<'mapper>>> {
        self.prev
    }

    fn set_next(&mut self, next: Option<NonNull<Segment<'mapper>>>) {
        self.next = next;
    }

    fn set_prev(&mut self, prev: Option<NonNull<Segment<'mapper>>>) {
        self.prev = prev;
    }
}
