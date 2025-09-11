use std::ptr::NonNull;

use getset::Getters;

use crate::{MapError, mapper::Mapper};

#[derive(Debug, Getters)]
pub struct Region<'mapper, M>
where
    M: Mapper + ?Sized,
{
    #[getset(get = "pub")]
    data: NonNull<[u8]>,
    mapper: &'mapper M,
    activate: bool,
}

impl<'mapper, M> Region<'mapper, M>
where
    M: Mapper + ?Sized,
{
    pub fn new(mapper: &'mapper M, size: usize) -> Result<Self, MapError> {
        let data = mapper.map(size)?;
        Ok(Self {
            data,
            mapper,
            activate: false,
        })
    }

    pub fn activate(&mut self) -> Result<(), MapError> {
        self.mapper.commit(self.data)?;
        self.activate = true;
        Ok(())
    }

    pub fn deactivate(&mut self) -> Result<(), MapError> {
        self.mapper.decommit(self.data)?;
        self.activate = false;
        Ok(())
    }

    pub fn as_ref(&self) -> Option<&[u8]> {
        if self.activate {
            Some(unsafe { self.data.as_ref() })
        } else {
            None
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut [u8]> {
        if self.activate {
            Some(unsafe { self.data.as_mut() })
        } else {
            None
        }
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
