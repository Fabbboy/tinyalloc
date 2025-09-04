pub mod classes;
pub mod segment;

const ONE_KB: usize = 1024;
const ONE_MB: usize = 1024 * ONE_KB;
pub const QUANTUM: usize = 16;
pub const CUT_OFF: usize = 64 * ONE_KB;
pub const SPAN_SIZE: usize = CUT_OFF;
pub const SEGMENT_SIZE: usize = 2 * ONE_MB;
