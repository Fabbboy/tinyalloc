use std::ptr::NonNull;

use anyhow::Result;
use enumset::{EnumSet, EnumSetType};

use crate::MapError;

#[derive(EnumSetType)]
pub enum Protection {
    Read,
    Write,
}

pub trait Mapper {
    fn cptr<T>(rptr: *mut T) -> *mut libc::c_void {
        rptr as *mut libc::c_void
    }

    fn map(size: usize) -> Result<NonNull<[u8]>> {
        _ = size;
        Err(MapError::OutOfMemory.into())
    }
    fn unmap(ptr: NonNull<[u8]>) {
        _ = ptr;
    }
    fn commit(ptr: NonNull<[u8]>) -> Result<()> {
        _ = ptr;
        Err(MapError::CommitFailed.into())
    }
    fn decommit(ptr: NonNull<[u8]>) -> Result<()> {
        _ = ptr;
        Ok(())
    }
    fn protect(ptr: NonNull<[u8]>, prot: EnumSet<Protection>) -> Result<()> {
        _ = (ptr, prot);
        Err(MapError::ProtectFailed.into())
    }
}
