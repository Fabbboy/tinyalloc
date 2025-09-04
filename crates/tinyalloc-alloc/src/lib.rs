pub mod arena;
pub mod classes;

const ONE_KB: usize = 1024;
pub const QUANTUM: usize = 16;
pub const CUT_OFF: usize = 64 * ONE_KB;
pub const SPAN_SIZE: usize = CUT_OFF;
