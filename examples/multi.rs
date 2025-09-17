use tinyalloc::TinyAlloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;
fn main() {}
