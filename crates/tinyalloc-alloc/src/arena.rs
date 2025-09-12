use core::slice;
use enumset::enum_set;
use std::ptr::NonNull;

use tinyalloc_sys::{
    MapError,
    mapper::{Mapper, Protection},
    region::Region,
};

use crate::{
    classes::class_init,
    config::{ArenaConfig, SIZES, WORD, align_up},
    queue::Queue,
};

#[derive(Debug)]
pub enum ArenaError {
    MapError(MapError),
    Insufficient,
}


pub struct Arena<'mapper, M>
where
    M: Mapper + ?Sized,
{
    config: ArenaConfig,
    region: Region<'mapper, M>,
    classes: [Queue<'mapper>; SIZES],
}

impl<'mapper, M> Arena<'mapper, M>
where
    M: Mapper + ?Sized,
{
    fn inner_new(config: ArenaConfig, region: Region<'mapper, M>) -> Self {
        let classes: [Queue; SIZES] = class_init(|class| Queue::new(class));

        Self {
            classes,
            config,
            region,
        }
    }


    pub fn new(config: &ArenaConfig, mapper: &'mapper M) -> Result<NonNull<Self>, ArenaError> {
        let region = Region::new(mapper, *config.arena_size()).map_err(ArenaError::MapError)?;

        let arena_size = core::mem::size_of::<Self>();
        let total_size = align_up(arena_size, WORD);

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

        let arena = Self::inner_new(config.clone(), region);

        let arena_ptr = base_ptr as *mut Self;
        unsafe {
            core::ptr::write(arena_ptr, arena);
        }

        Ok(NonNull::new(arena_ptr).unwrap())
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
