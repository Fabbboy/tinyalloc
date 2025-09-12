use std::{num::NonZeroUsize, ptr::NonNull};

use enumset::{EnumSet, EnumSetType};

use crate::MapError;

#[derive(EnumSetType)]
pub enum Protection {
    Read,
    Write,
}

pub trait MapperRequires
where
    Self: Send + Sync,
{
}

pub trait Mapper
where
    Self: MapperRequires,
{
    fn cptr(&self, rptr: *mut u8) -> *mut libc::c_void {
        rptr as *mut libc::c_void
    }
    fn map(&self, size: NonZeroUsize) -> Result<NonNull<[u8]>, MapError> {
        _ = size;
        Err(MapError::OutOfMemory)
    }
    fn unmap(&self, ptr: NonNull<[u8]>) {
        _ = ptr;
    }
    fn decommit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
        _ = ptr;
        Ok(())
    }
    fn protect(&self, ptr: NonNull<[u8]>, prot: EnumSet<Protection>) -> Result<(), MapError> {
        _ = (ptr, prot);
        Err(MapError::ProtectFailed)
    }
}
