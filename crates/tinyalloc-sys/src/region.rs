use std::{num::NonZeroUsize, ptr::NonNull};

use enumset::EnumSet;
use getset::Getters;

use crate::{
    MapError,
    mapper::{Mapper, Protection},
};

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
    pub fn new(mapper: &'mapper M, size: NonZeroUsize) -> Result<Self, MapError> {
        let data = mapper.map(size)?;
        Ok(Self {
            data,
            mapper,
            activate: false,
        })
    }

    pub fn activate(&mut self) -> Result<(), MapError> {
        self.mapper.protect(self.data, EnumSet::all())?;
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

    pub fn partial(
        &self,
        range: NonNull<[u8]>,
        protection: EnumSet<Protection>,
    ) -> Result<(), MapError> {
        if protection.is_empty() {
            self.mapper.decommit(range)?;
        }

        self.mapper.protect(range, protection)?;
        Ok(())
    }

    pub fn as_mut(&mut self) -> Option<&mut [u8]> {
        if self.activate {
            Some(unsafe { self.data.as_mut() })
        } else {
            None
        }
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.data.as_ptr() as *mut u8
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
