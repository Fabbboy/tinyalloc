use tinyalloc_list::List;
use tinyalloc_sys::mapper::Mapper;

use crate::arena::Arena;

pub struct Heap<'mapper, M>
where
    M: Mapper,
{
    mapper: &'mapper M,
    arenas: List<Arena<'mapper, M>>,
}

impl<'mapper, M> Heap<'mapper, M>
where
    M: Mapper,
{
    pub fn new(mapper: &'mapper M) -> Self {
        Self {
            mapper,
            arenas: List::new(),
        }
    }
}
