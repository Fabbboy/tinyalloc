use std::ptr::NonNull;

use anyhow::Result;
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
    fn map(&self, size: usize) -> Result<NonNull<[u8]>> {
        _ = size;
        Err(MapError::OutOfMemory.into())
    }
    fn unmap(&self, ptr: NonNull<[u8]>) {
        _ = ptr;
    }
    fn commit(&self, ptr: NonNull<[u8]>) -> Result<()> {
        _ = ptr;
        Err(MapError::CommitFailed.into())
    }
    fn decommit(&self, ptr: NonNull<[u8]>) -> Result<()> {
        _ = ptr;
        Ok(())
    }
    fn protect(&self, ptr: NonNull<[u8]>, prot: EnumSet<Protection>) -> Result<()> {
        _ = (ptr, prot);
        Err(MapError::ProtectFailed.into())
    }
}
