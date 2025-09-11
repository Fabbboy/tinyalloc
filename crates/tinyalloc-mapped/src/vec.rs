use std::ptr::NonNull;

use anyhow::Result;
use tinyalloc_sys::{mapper::Mapper, region::Region};
use tinyvec::SliceVec;

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
    T: Copy + Default,
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
        backing.activate()?;
        let raw_slice: NonNull<[u8]> = *backing.data();
        let slice = Self::tslice(raw_slice);
        let vec = SliceVec::from(slice);
        Ok(Self {
            vec,
            backing: Some(backing),
        })
    }

    fn resize_backing(&mut self, new_capacity: usize) -> Result<()> {
        if new_capacity == 0 {
            self.backing = None;
            self.vec = SliceVec::default();
            return Ok(());
        }

        let new_backing = Region::<M>::new(new_capacity * Self::T_SIZE)?;
        new_backing.activate()?;
        let raw_slice: NonNull<[u8]> = *new_backing.data();
        let new_slice = Self::tslice(raw_slice);

        let mut new_vec = SliceVec::from(new_slice);
        for item in self.vec.as_slice() {
            new_vec.push(*item);
        }

        self.vec = new_vec;
        self.backing = Some(new_backing);
        Ok(())
    }

    fn ensure_capacity(&mut self, additional: usize) -> Result<()> {
        let required = self.vec.len() + additional;
        if required <= self.vec.capacity() {
            return Ok(());
        }

        let current_capacity = self.vec.capacity();
        let new_capacity = if current_capacity == 0 {
            required.max(1)
        } else {
            (current_capacity * Self::GROWTH).max(required)
        };
        self.resize_backing(new_capacity)
    }

    fn shrink_if_needed(&mut self) -> Result<()> {
        let current_len = self.vec.len();
        let current_capacity = self.vec.capacity();

        if current_len == 0 || current_capacity == 0 {
            return Ok(());
        }

        let usage_ratio = current_len as f64 / current_capacity as f64;
        if usage_ratio < Self::SHRINK && current_capacity > 1 {
            let new_capacity = (current_capacity / Self::GROWTH).max(current_len);
            self.resize_backing(new_capacity)?;
        }
        Ok(())
    }

    pub fn push(&mut self, value: T) -> Result<()> {
        self.ensure_capacity(1)?;
        self.vec.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let result = self.vec.pop();
        if result.is_some() {
            let _ = self.shrink_if_needed();
        }
        result
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
        let _ = self.shrink_if_needed();
    }

    pub fn insert(&mut self, index: usize, element: T) -> Result<()> {
        self.ensure_capacity(1)?;
        self.vec.insert(index, element);
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> T {
        let result = self.vec.remove(index);
        let _ = self.shrink_if_needed();
        result
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        let result = self.vec.swap_remove(index);
        let _ = self.shrink_if_needed();
        result
    }

    pub fn truncate(&mut self, len: usize) {
        self.vec.truncate(len);
        let _ = self.shrink_if_needed();
    }

    pub fn as_slice(&self) -> &[T] {
        self.vec.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.vec.as_mut_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinyalloc_sys::posix::PosixMapper;

    type TestVec<T> = MappedVec<'static, T, PosixMapper>;

    #[test]
    fn test_basic_operations() {
        let mut vec: TestVec<i32> = MappedVec::new();
        assert!(vec.is_empty());

        vec.push(42).unwrap();
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.as_slice(), &[42]);

        assert_eq!(vec.pop(), Some(42));
        assert!(vec.is_empty());
    }

    #[test]
    fn test_capacity_growth() {
        let mut vec: TestVec<i32> = MappedVec::new_capacity(2).unwrap();
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap(); // Should trigger growth
        assert!(vec.capacity() >= 3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_clear_and_shrink() {
        let mut vec: MappedVec<i32, PosixMapper> = MappedVec::new_capacity(10).unwrap();
        for i in 0..10 {
            vec.push(i).unwrap();
        }
        vec.clear();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_insert_remove() {
        let mut vec: MappedVec<i32, PosixMapper> = MappedVec::new();
        vec.push(1).unwrap();
        vec.push(3).unwrap();
        vec.insert(1, 2).unwrap();
        assert_eq!(vec.as_slice(), &[1, 2, 3]);

        let removed = vec.remove(1);
        assert_eq!(removed, 2);
        assert_eq!(vec.as_slice(), &[1, 3]);
    }

    #[test]
    fn test_different_types() {
        let mut vec_u8: MappedVec<u8, PosixMapper> = MappedVec::new();
        vec_u8.push(255u8).unwrap();
        assert_eq!(vec_u8.as_slice(), &[255u8]);

        let mut vec_f64: MappedVec<f64, PosixMapper> = MappedVec::new();
        vec_f64.push(3.14).unwrap();
        assert_eq!(vec_f64.as_slice(), &[3.14]);
    }

    #[test]
    fn test_slice_access() {
        let mut vec: MappedVec<i32, PosixMapper> = MappedVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();

        let slice = vec.as_slice();
        assert_eq!(slice, &[1, 2]);

        let mut_slice = vec.as_mut_slice();
        mut_slice[0] = 10;
        assert_eq!(vec.as_slice(), &[10, 2]);
    }
}
