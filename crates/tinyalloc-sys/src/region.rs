use std::{marker::PhantomData, ptr::NonNull};

use anyhow::Result;
use getset::Getters;

use crate::mapper::Mapper;

#[derive(Debug, Getters)]
pub struct Region<M>
where
    M: Mapper,
{
    #[getset(get = "pub")]
    data: NonNull<[u8]>,
    _marker: PhantomData<M>,
}

impl<M> Region<M>
where
    M: Mapper,
{
    pub fn new(size: usize) -> Result<Self> {
        let data = M::map(size)?;
        Ok(Self {
            data,
            _marker: PhantomData,
        })
    }

    pub fn activate(&self) -> Result<()> {
        M::commit(self.data)
    }

    pub fn deactivate(&self) -> Result<()> {
        M::decommit(self.data)
    }
}

impl<M> Drop for Region<M>
where
    M: Mapper,
{
    fn drop(&mut self) {
        M::unmap(self.data);
    }
}
