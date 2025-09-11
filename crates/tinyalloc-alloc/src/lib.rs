pub mod arena;
pub mod classes;
pub mod heap;

pub const SIZES: usize = 44;
pub const WORD: usize = std::mem::size_of::<usize>();
