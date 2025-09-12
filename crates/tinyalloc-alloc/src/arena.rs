use core::slice;
use enumset::enum_set;
use std::{num::NonZeroUsize, ptr::NonNull};

use tinyalloc_sys::{
    MapError,
    mapper::{Mapper, Protection},
    region::Region,
};

use crate::{
    classes::class_init,
    config::{SIZES, WORD, align_up},
    queue::Queue,
};

#[derive(Debug)]
pub enum ArenaError {
    MapError(MapError),
    Insufficient,
    SizeIsZero,
}

pub struct Arena<'mapper, M>
where
    M: Mapper + ?Sized,
{
    region: Region<'mapper, M>,
    classes: [Queue<'mapper>; SIZES],
}

impl<'mapper, M> Arena<'mapper, M>
where
    M: Mapper + ?Sized,
{
    fn inner_new(region: Region<'mapper, M>) -> Self {
        let classes: [Queue<'mapper>; SIZES] = class_init(|class| Queue::new(class));

        Self { classes, region }
    }

    pub fn new(size: usize, mapper: &'mapper M) -> Result<NonNull<Self>, ArenaError> {
        let nonz = NonZeroUsize::new(size).ok_or(ArenaError::SizeIsZero)?;
        let region = Region::new(mapper, nonz).map_err(ArenaError::MapError)?;

        let arena_size = core::mem::size_of::<Self>();
        let total_size = align_up(arena_size, WORD);

        if total_size >= nonz.get() {
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

        let arena = Self::inner_new(region);

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
    use tinyalloc_sys::GLOBAL_MAPPER;

    #[test]
    fn test_arena_construction() {
        let arena_result = Arena::new(ARENA_INITIAL_SIZE, GLOBAL_MAPPER);
        assert!(arena_result.is_ok());
    }

    #[test]
    fn test_arena_insufficient_space() {
        let arena_result = Arena::new(1, GLOBAL_MAPPER);
        assert!(matches!(arena_result, Err(ArenaError::Insufficient)));
    }
}
