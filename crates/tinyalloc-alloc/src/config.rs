use std::num::NonZeroUsize;

use getset::Getters;

pub const SIZES: usize = 32;
pub const ONE: usize = 1;
pub const WORD: usize = core::mem::size_of::<usize>();

pub const SHIFT: usize = match WORD {
    8 => 3,
    4 => 2,
    _ => panic!("Unsupported word size"),
};

pub const MAX_ALIGN: usize = 1 << (SIZES - SHIFT - 1);
pub const MAX_SIZE: usize = MAX_ALIGN << (SIZES - SHIFT - 1);
pub const MIN_ALIGN: usize = WORD;
pub const MIN_SIZE: usize = MIN_ALIGN;

pub const ARENA_SHIFT: usize = 23 + SHIFT;
pub const ARENA_SIZE: usize = 1 << ARENA_SHIFT;

pub const SEGMENT_SHIFT: usize = 13 + SHIFT;
pub const SEGMENT_SIZE: usize = 1 << SEGMENT_SHIFT;

pub const SMALL_SC_LIMIT: usize = 1 << (SHIFT + 5);
pub const MEDIUM_SC_LIMIT: usize = 1 << (SHIFT + 10);
pub const LARGE_SC_LIMIT: usize = 1 << (SHIFT + 13);

pub const SMALL_ALIGN_LIMIT: usize = SMALL_SC_LIMIT / 4;
pub const MEDIUM_ALIGN_LIMIT: usize = MEDIUM_SC_LIMIT / 8;
pub const LARGE_ALIGN_RATIO: usize = 8;

pub const fn align_up(size: usize, align: usize) -> usize {
    (size + align - 1) & !(align - 1)
}

#[derive(Getters, Clone)]
pub struct SegmentConfig {
    #[getset(get = "pub")]
    size: NonZeroUsize,
}

impl SegmentConfig {
    pub fn new(size: NonZeroUsize) -> SegmentConfig {
        SegmentConfig { size }
    }
}

#[derive(Getters, Clone)]
pub struct ArenaConfig {
    #[getset(get = "pub")]
    arena_size: NonZeroUsize,
    #[getset(get = "pub")]
    segment_config: SegmentConfig,
}

impl ArenaConfig {
    pub fn new(arena_size: NonZeroUsize, segment_config: &SegmentConfig) -> ArenaConfig {
        ArenaConfig {
            arena_size,
            segment_config: segment_config.clone(),
        }
    }
}
