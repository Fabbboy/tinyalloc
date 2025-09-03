use std::ptr::NonNull;

use getset::Getters;

use crate::vm::{MapError, Mapper};

#[derive(Getters)]
pub struct Page<'mapper> {
    mapper: &'mapper dyn Mapper,
    #[getset(get = "pub")]
    ptr: NonNull<[u8]>,
}

impl<'mapper> Page<'mapper> {
    pub fn new(mapper: &'mapper dyn Mapper, size: usize) -> Result<Self, MapError> {
        let ptr = mapper.map(size)?;
        Ok(Self { mapper, ptr })
    }

    pub fn as_ptr(&mut self) -> NonNull<u8> {
        unsafe { NonNull::new_unchecked(self.ptr.as_mut().as_mut_ptr()) }
    }

    pub fn len(&self) -> usize {
        self.ptr.len()
    }

    pub fn commit(&mut self) -> Result<(), MapError> {
        self.mapper.commit(self.as_ptr(), self.len())
    }

    pub fn decommit(&mut self) -> Result<(), MapError> {
        self.mapper.decommit(self.as_ptr(), self.len())
    }

    pub fn protect(&mut self) -> Result<(), MapError> {
        self.mapper.protect(self.as_ptr(), self.len())
    }

    pub fn unprotect(&mut self) -> Result<(), MapError> {
        self.mapper.unprotect(self.as_ptr(), self.len())
    }
}
