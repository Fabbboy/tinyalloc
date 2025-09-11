use std::ptr::NonNull;

use anyhow::Result;
use getset::Getters;

use crate::mapper::Mapper;

#[derive(Debug, Getters)]
pub struct Region<'mapper, M>
where
    M: Mapper + ?Sized,
{
    #[getset(get = "pub")]
    data: NonNull<[u8]>,
    mapper: &'mapper M,
}

impl<'mapper, M> Region<'mapper, M>
where
    M: Mapper + ?Sized,
{
    pub fn new(mapper: &'mapper M, size: usize) -> Result<Self> {
        let data = mapper.map(size)?;
        Ok(Self { data, mapper })
    }

    pub fn activate(&self) -> Result<()> {
        self.mapper.commit(self.data)
    }

    pub fn deactivate(&self) -> Result<()> {
        self.mapper.decommit(self.data)
    }
}

impl<'mapper, M> Drop for Region<'mapper, M>
where
    M: Mapper + ?Sized,
{
    fn drop(&mut self) {
        self.mapper.unmap(self.data);
    }
}
