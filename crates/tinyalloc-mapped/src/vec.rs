use std::ptr::NonNull;

use anyhow::Result;
use thiserror::Error;
use tinyalloc_sys::{MapError, mapper::Mapper, region::Region};
use tinyvec::SliceVec;

#[derive(Debug, Error)]
pub enum MappedVecError {
    #[error("mapping error: {0}")]
    MapError(#[from] MapError),
}

#[derive(Debug)]
pub struct MappedVec<'vec, T, M>
where
    M: Mapper,
{
    vec: SliceVec<'vec, T>,
    backing: Option<Region<M>>,
}

impl<'vec, T, M> Default for MappedVec<'vec, T, M>
where
    M: Mapper,
{
    fn default() -> Self {
        Self {
            vec: SliceVec::default(),
            backing: None,
        }
    }
}

impl<'vec, T, M> MappedVec<'vec, T, M>
where
    M: Mapper,
{
    const T_SIZE: usize = std::mem::size_of::<T>();
    const GROWTH: usize = 2;
    const SHRINK: f64 = 0.25;

    const fn tslice<'inner>(raw: NonNull<[u8]>) -> &'inner mut [T] {
        let raw_ptr = raw.as_ptr() as *mut T;
        let len = raw.len() / Self::T_SIZE;
        unsafe { std::slice::from_raw_parts_mut(raw_ptr, len) }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_capacity(initial: usize) -> Result<Self> {
        let backing = Region::<M>::new(initial * Self::T_SIZE)?;
        let raw_slice: NonNull<[u8]> = *backing.data();
        let slice = Self::tslice(raw_slice);
        let vec = SliceVec::from(slice);
        Ok(Self {
            vec,
            backing: Some(backing),
        })
    }
}
