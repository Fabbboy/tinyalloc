use std::ptr::NonNull;

pub struct MapError;

pub trait Mapper {
    fn map(&self, size: usize) -> Result<NonNull<[u8]>, MapError>;
    fn unmap(&self, ptr: NonNull<u8>, size: usize);
    fn commit(&self, ptr: NonNull<u8>, size: usize) -> Result<(), MapError>;
    fn decommit(&self, ptr: NonNull<u8>, size: usize) -> Result<(), MapError>;
    fn protect(&self, ptr: NonNull<u8>, size: usize) -> Result<(), MapError>;
    fn unprotect(&self, ptr: NonNull<u8>, size: usize) -> Result<(), MapError>;
}
