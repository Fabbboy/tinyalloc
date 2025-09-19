pub const SIZES: usize = 84;
pub const ONE: usize = 1;
pub const WORD: usize = core::mem::size_of::<usize>();


pub const SHIFT: usize = WORD.trailing_zeros() as usize;
pub const MIN_ALIGN: usize = WORD;
pub const MIN_SIZE: usize = MIN_ALIGN;

pub const ARENA_SHIFT: usize = 23 + SHIFT;
pub const ARENA_INITIAL_SIZE: usize = 1 << ARENA_SHIFT;
pub const ARENA_GROWTH: usize = 2;
pub const ARENA_STEP: usize = 4;
pub const ARENA_LIMIT: usize = 80;

pub const SEGMENT_SHIFT: usize = 16 + SHIFT;
pub const SEGMENT_SIZE: usize = 1 << SEGMENT_SHIFT;

pub const SMALL_SC_LIMIT: usize = 1 << (SHIFT + 5);
pub const MEDIUM_SC_LIMIT: usize = 1 << (SHIFT + 10);
pub const LARGE_SC_LIMIT: usize = 1 << (SHIFT + 15);

pub const SMALL_ALIGN_LIMIT: usize = SMALL_SC_LIMIT / 4;
pub const MEDIUM_ALIGN_LIMIT: usize = MEDIUM_SC_LIMIT / 8;
pub const LARGE_ALIGN_RATIO: usize = 8;

pub const SMALL_ALIGN_CLASSES: usize = SMALL_ALIGN_LIMIT / MIN_ALIGN;
pub const SMALL_RATIO: usize = SMALL_SC_LIMIT / SMALL_ALIGN_LIMIT;

pub const REMOTE_BATCH_SIZE: usize = 32;
pub const REMOTE_CHECK_FREQUENCY: usize = 16;
pub const REMOTE_MAX_BATCH: usize = 64;

pub const QUEUE_THRESHOLD: usize = 12;

