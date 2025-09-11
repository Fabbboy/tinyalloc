use tinyalloc_sys::mapper::Mapper;

pub struct Heap<'mapper, M>
where
    M: Mapper,
{
    mapper: &'mapper M,
}

impl<'mapper, M> Heap<'mapper, M>
where
    M: Mapper,
{
    pub const fn new(mapper: &'mapper M) -> Self {
        Self { mapper }
    }
}
