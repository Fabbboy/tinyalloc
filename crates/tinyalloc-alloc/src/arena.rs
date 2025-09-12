use core::slice;
use enumset::enum_set;
use std::ptr::NonNull;

use tinyalloc_bitmap::Bitmap;
use tinyalloc_sys::{
    MapError,
    mapper::{Mapper, Protection},
    region::Region,
};

use crate::config::ArenaConfig;

#[derive(Debug)]
pub enum ArenaError {
    MapError(MapError),
    Insufficient,
}

pub struct Arena<'mapper, M>
where
    M: Mapper,
{
    config: ArenaConfig,
    region: Region<'mapper, M>,
    bitmap: Bitmap<'mapper, usize>,
}

impl<'mapper, M> Arena<'mapper, M>
where
    M: Mapper,
{
    fn inner_new(
        config: ArenaConfig,
        region: Region<'mapper, M>,
        bitmap: Bitmap<'mapper, usize>,
    ) -> Self {
        Self {
            config,
            region,
            bitmap,
        }
    }

    fn header_layout(config: &ArenaConfig) -> (usize, usize, usize, usize) {
        let arena_size = core::mem::size_of::<Self>();
        let segment_size = config.segment_config().size().get();
        let segments = config.arena_size().get() / segment_size;
        let bm_words = Bitmap::<usize>::words(segments);
        let bm_bytes = bm_words * core::mem::size_of::<usize>();
        let total_size = arena_size + bm_bytes;

        (arena_size, segments, bm_words, total_size)
    }

    fn create_bitmap(
        base_ptr: *mut u8,
        arena_size: usize,
        segments: usize,
        bm_words: usize,
    ) -> Bitmap<'mapper, usize> {
        let bitmap_ptr = unsafe { base_ptr.add(arena_size) as *mut usize };
        let bitmap_slice = unsafe { slice::from_raw_parts_mut(bitmap_ptr, bm_words) };

        let mut bitmap = Bitmap::within(bitmap_slice, segments).unwrap();
        bitmap.clear_all();
        bitmap
    }

    pub fn new(config: &ArenaConfig, mapper: &'mapper M) -> Result<NonNull<Self>, ArenaError> {
        let region = Region::new(mapper, *config.arena_size()).map_err(ArenaError::MapError)?;

        let (arena_size, segments, bm_words, total_size) = Self::header_layout(config);

        if total_size >= config.arena_size().get() {
            return Err(ArenaError::Insufficient);
        }

        let base_ptr = region.as_ptr();
        let activation_slice = unsafe { slice::from_raw_parts_mut(base_ptr, total_size) };
        let activation_range = NonNull::new(activation_slice as *mut [u8]).unwrap();

        region
            .partial(
                activation_range,
                enum_set!(Protection::Read | Protection::Write),
            )
            .map_err(ArenaError::MapError)?;

        let bitmap = Self::create_bitmap(base_ptr, arena_size, segments, bm_words);
        let arena = Self::inner_new(config.clone(), region, bitmap);

        let arena_ptr = base_ptr as *mut Self;
        unsafe {
            core::ptr::write(arena_ptr, arena);
        }

        Ok(NonNull::new(arena_ptr).unwrap())
    }
}
