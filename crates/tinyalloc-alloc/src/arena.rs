use std::ptr::NonNull;

use tinyalloc_list::Item;
use tinyalloc_sys::{MapError, mapper::Mapper, region::Region};

pub struct Arena<'mapper, M>
where
    M: Mapper,
{
    mapper: &'mapper M,
    region: Region<'mapper, M>,

    next: Option<NonNull<Self>>,
    prev: Option<NonNull<Self>>,
}

impl<'mapper, M> Arena<'mapper, M>
where
    M: Mapper,
{
    pub fn new(mapper: &'mapper M, size: usize) -> Result<Self, MapError> {
        let region = Region::new(mapper, size)?;
        Ok(Self {
            mapper,
            region,
            next: None,
            prev: None,
        })
    }
}

impl<'mapper, M> Item for Arena<'mapper, M>
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