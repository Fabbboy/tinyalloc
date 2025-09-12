use std::ptr::NonNull;

use tinyalloc_list::List;

use crate::{classes::Class, segment::Segment};

pub struct Queue<'mapper> {
    class: &'static Class,
    free_list: List<Segment<'mapper>>,
    partial_list: List<Segment<'mapper>>,
    full_list: List<Segment<'mapper>>,
}

impl<'mapper> Queue<'mapper> {
    pub fn new(class: &'static Class) -> Queue<'mapper> {
        Queue {
            class,
            free_list: List::new(),
            partial_list: List::new(),
            full_list: List::new(),
        }
    }
}
