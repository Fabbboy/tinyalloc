use std::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

use crate::{classes::class_init, config::SIZES, queue::Queue};

pub struct Heap<'mapper> {
    classes: [Queue<'mapper>; SIZES],
}

impl<'mapper> Heap<'mapper> {
    pub fn new() -> Self {
        let classes: [Queue<'mapper>; SIZES] = class_init(|class| Queue::new(class));
        Self { classes }
    }

    pub fn allocate(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        todo!()
    }
    pub fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
      todo!()
    }
}

unsafe impl<'mapper> GlobalAlloc for Heap<'mapper> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}