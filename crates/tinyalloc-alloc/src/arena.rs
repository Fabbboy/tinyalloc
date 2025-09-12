use core::slice;
use enumset::enum_set;
use std::ptr::NonNull;

use tinyalloc_bitmap::Bitmap;
use tinyalloc_sys::{
    MapError,
    mapper::{Mapper, Protection},
    region::Region,
};

use crate::config::{ArenaConfig, WORD, align_up};

#[derive(Debug)]
pub enum ArenaError {
    MapError(MapError),
    Insufficient,
}

struct HeaderLayout {
    segments: usize,
    bm_words: usize,
    bitmap_offset: usize,
    total_size: usize,
}

pub struct Arena<'mapper, M>
where
    M: Mapper + ?Sized,
{
    config: ArenaConfig,
    region: Region<'mapper, M>,
    bitmap: Bitmap<'mapper, usize>,
}

impl<'mapper, M> Arena<'mapper, M>
where
    M: Mapper + ?Sized,
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

    fn header_layout(config: &ArenaConfig) -> HeaderLayout {
        let arena_size = core::mem::size_of::<Self>();
        let segment_size = config.segment_config().size().get();
        let segments = config.arena_size().get() / segment_size;
        let bm_words = Bitmap::<usize>::words(segments);
        let bm_bytes = bm_words * core::mem::size_of::<usize>();

        let aligned_bitmap_offset = align_up(arena_size, WORD);
        let total_size = aligned_bitmap_offset + bm_bytes;

        HeaderLayout {
            segments,
            bm_words,
            bitmap_offset: aligned_bitmap_offset,
            total_size,
        }
    }

    fn create_bitmap(
        base_ptr: *mut u8,
        aligned_bitmap_offset: usize,
        segments: usize,
        bm_words: usize,
    ) -> Bitmap<'mapper, usize> {
        let bitmap_ptr = unsafe { base_ptr.add(aligned_bitmap_offset) as *mut usize };
        let bitmap_slice = unsafe { slice::from_raw_parts_mut(bitmap_ptr, bm_words) };

        let mut bitmap = Bitmap::within(bitmap_slice, segments).unwrap();
        bitmap.clear_all();
        bitmap
    }

    pub fn new(config: &ArenaConfig, mapper: &'mapper M) -> Result<NonNull<Self>, ArenaError> {
        let region = Region::new(mapper, *config.arena_size()).map_err(ArenaError::MapError)?;

        let layout = Self::header_layout(config);

        if layout.total_size >= config.arena_size().get() {
            return Err(ArenaError::Insufficient);
        }

        let base_ptr = region.as_ptr();
        let activation_slice = unsafe { slice::from_raw_parts_mut(base_ptr, layout.total_size) };
        let activation_range = NonNull::new(activation_slice as *mut [u8]).unwrap();

        region
            .partial(
                activation_range,
                enum_set!(Protection::Read | Protection::Write),
            )
            .map_err(ArenaError::MapError)?;

        let bitmap = Self::create_bitmap(
            base_ptr,
            layout.bitmap_offset,
            layout.segments,
            layout.bm_words,
        );
        let arena = Self::inner_new(config.clone(), region, bitmap);

        let arena_ptr = base_ptr as *mut Self;
        unsafe {
            core::ptr::write(arena_ptr, arena);
        }

        Ok(NonNull::new(arena_ptr).unwrap())
    }

    pub fn is_empty(&self) -> bool {
        self.bitmap.is_clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use std::num::NonZeroUsize;
    use tinyalloc_sys::GLOBAL_MAPPER;

    #[test]
    fn test_arena_construction() {
        let segment_config = SegmentConfig::new(NonZeroUsize::new(SEGMENT_SIZE).unwrap());
        let arena_config = ArenaConfig::new(
            NonZeroUsize::new(ARENA_INITIAL_SIZE).unwrap(),
            &segment_config,
        );

        let arena_result = Arena::new(&arena_config, GLOBAL_MAPPER);
        assert!(arena_result.is_ok());

        let layout = Arena::<dyn Mapper>::header_layout(&arena_config);
        assert_eq!(layout.bitmap_offset % WORD, 0);
    }

    #[test]
    fn test_arena_insufficient_space() {
        let segment_config = SegmentConfig::new(NonZeroUsize::new(SEGMENT_SIZE).unwrap());
        let arena_config = ArenaConfig::new(
            NonZeroUsize::new(ARENA_INITIAL_SIZE).unwrap(),
            &segment_config,
        );

        let arena_result = Arena::new(&arena_config, GLOBAL_MAPPER);
        assert!(matches!(arena_result, Err(ArenaError::Insufficient)));
    }
}
