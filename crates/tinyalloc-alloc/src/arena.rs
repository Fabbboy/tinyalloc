use std::{
    array, mem,
    ptr::{self, NonNull},
    slice,
};

use tinyalloc_bitmap::{Bitmap, BitmapError};
use tinyalloc_list::{Item, List};
use tinyalloc_sys::{MapError, mapper::Mapper, region::Region};

use crate::{
    ARENA_SIZE, SEGMENT_SIZE, SIZES,
    segment::{Segment, SegmentId},
};

#[derive(Debug)]
pub enum ArenaError {
    Full,
    MapError(MapError),
    BitmapError(BitmapError),
}

pub struct Arena<'mapper, M>
where
    M: Mapper,
{
    region: Region<'mapper, M>,
    next: Option<NonNull<Self>>,
    prev: Option<NonNull<Self>>,
    segment_map: Bitmap<'mapper, usize>,
    segments: [List<Segment<'mapper, M>>; SIZES],
}

impl<'mapper, M> Arena<'mapper, M>
where
    M: Mapper,
{
    pub fn new(mapper: &'mapper M) -> Result<NonNull<Self>, ArenaError> {
        let mut region = Region::new(mapper, ARENA_SIZE).map_err(ArenaError::MapError)?;
        let data = region.as_mut().unwrap();
        let arena_ptr = data.as_mut_ptr() as *mut Self;
        let arena_size = mem::size_of::<Self>();
        let num_segments = ARENA_SIZE / SEGMENT_SIZE;

        let segments = array::from_fn(|_| List::new());

        let bitmap_words = Bitmap::<usize>::words(num_segments);
        let bitmap_start = unsafe { arena_ptr.byte_add(arena_size) as *mut usize };
        let bitmap_slice = unsafe { slice::from_raw_parts_mut(bitmap_start, bitmap_words) };

        let mut segment_map =
            Bitmap::within(bitmap_slice, num_segments).map_err(ArenaError::BitmapError)?;
        segment_map.clear_all();

        unsafe {
            ptr::write(
                arena_ptr,
                Self {
                    region,
                    next: None,
                    prev: None,
                    segment_map,
                    segments,
                },
            );
        }

        Ok(NonNull::new(arena_ptr).unwrap())
    }

    fn translate(&mut self, index: usize) -> *mut Segment<'mapper, M> {
        let slice = self.region.as_mut().unwrap();
        let base = slice.as_mut_ptr() as usize;
        let offset = index * SEGMENT_SIZE;
        (base + offset) as *mut Segment<'mapper, M>
    }

    pub(crate) fn allocate(
        &mut self,
    ) -> Result<(NonNull<Segment<'mapper, M>>, SegmentId), ArenaError> {
        let index = match self.segment_map.find_first_clear() {
            Some(i) => i,
            None => return Err(ArenaError::Full),
        };
        let segment_ptr = self.translate(index);
        self.segment_map
            .set(index)
            .map_err(ArenaError::BitmapError)?;
        Ok((NonNull::new(segment_ptr).unwrap(), SegmentId(index)))
    }

    pub(crate) fn deallocate(&mut self, id: SegmentId) -> Result<(), ArenaError> {
        self.segment_map
            .clear(id.0)
            .map_err(ArenaError::BitmapError)
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
