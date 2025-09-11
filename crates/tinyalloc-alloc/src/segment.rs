use std::{marker::PhantomData, ptr::NonNull};

use tinyalloc_list::Item;
use tinyalloc_sys::mapper::Mapper;

use crate::{
    SEGMENT_SIZE,
    arena::{Arena, ArenaError},
    classes::Class,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentId(pub usize);

#[derive(Debug)]
pub enum SegmentError {
    ArenaError(ArenaError),
}

pub struct Segment<'mapper, M>
where
    M: Mapper,
{
    id: SegmentId,
    class: &'static Class,
    data: &'mapper mut [u8],
    arena: &'mapper mut Arena<'mapper, M>,
    _marker: PhantomData<M>,
    next: Option<NonNull<Self>>,
    prev: Option<NonNull<Self>>,
}

impl<'mapper, M> Segment<'mapper, M>
where
    M: Mapper,
{
    pub fn new(
        arena: &'mapper mut Arena<'mapper, M>,
        class: &'static Class,
    ) -> Result<NonNull<Self>, SegmentError> {
        let (mut segment_nn, id) = arena.allocate().map_err(SegmentError::ArenaError)?;
        let segment_ptr = segment_nn.as_ptr() as *mut u8;
        let segment_slice = unsafe { std::slice::from_raw_parts_mut(segment_ptr, SEGMENT_SIZE) };

        let segment = unsafe { segment_nn.as_mut() };
        segment.id = id.clone();
        segment.class = class;
        segment.data = segment_slice;
        segment._marker = PhantomData;
        segment.arena = arena;
        segment.next = None;
        segment.prev = None;
        Ok(segment_nn)
    }
}

impl<'mapper, M> Drop for Segment<'mapper, M>
where
    M: Mapper,
{
    fn drop(&mut self) {
        let _ = self.arena.deallocate(self.id);
    }
}

impl<'mapper, M> Item for Segment<'mapper, M>
where
    M: Mapper,
{
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }

    fn prev(&self) -> Option<NonNull<Self>> {
        self.prev
    }

    fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
        self.prev = prev;
    }
}
