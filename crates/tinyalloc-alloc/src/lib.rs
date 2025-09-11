pub mod arena;
pub mod classes;
pub mod heap;

pub const SIZES: usize = 44;
pub const ONE: usize = 1;
pub const WORD: usize = std::mem::size_of::<usize>();

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

pub const SEGMENT_NUM: usize = ARENA_SIZE / SEGMENT_SIZE;