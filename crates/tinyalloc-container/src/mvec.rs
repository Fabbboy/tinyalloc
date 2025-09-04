use std::{
  cell::OnceCell,
  mem,
  ptr::{
    self,
    NonNull,
  },
};

use tinyalloc_sys::{
  page::Page,
  size::page_size,
  vm::{
    MapError,
    Mapper,
  },
};

use crate::mqueue::MappedQueue;

pub const MVEC_GROWTH_FACTOR: usize = 2;
pub const MVEC_DECOMMIT_THRESHOLD: f32 = 0.25;
pub const MVEC_UNMAP_THRESHOLD: f32 = 0.125;

#[derive(Debug)]
pub enum MappedVectorError {
  MapError(MapError),
  IndexOutOfBounds,
}

impl From<MapError> for MappedVectorError {
  fn from(err: MapError) -> Self {
    MappedVectorError::MapError(err)
  }
}

pub struct MappedVector<'mapper, T, const N: usize = 10, const Q: usize = 8> {
  data: Option<NonNull<T>>,
  len: usize,
  capacity: usize,
  active_elements: usize,
  backing: MappedQueue<'mapper, Page<'mapper>, Q>,
  mapper: &'mapper dyn Mapper,
}

impl<'mapper, T, const N: usize, const Q: usize>
  MappedVector<'mapper, T, N, Q>
{
  const PAGE_SIZE: OnceCell<usize> = OnceCell::new();

  pub fn new(mapper: &'mapper dyn Mapper) -> Self {
    Self {
      data: None,
      len: 0,
      capacity: 0,
      active_elements: 0,
      backing: MappedQueue::new(mapper),
      mapper,
    }
  }

  fn elements_per_page(ps: usize) -> usize {
    ps / mem::size_of::<T>()
  }

  fn initial_capacity(&self) -> usize {
    let ps = *Self::PAGE_SIZE.get_or_init(|| page_size());
    // Use N as page multiplier, same as MappedQueue
    // This determines how many pages worth of elements we can store
    N * Self::elements_per_page(ps)
  }

  fn grow(&mut self, min_capacity: usize) -> Result<(), MappedVectorError> {
    if self.capacity >= min_capacity {
      return Ok(());
    }

    let ps = *Self::PAGE_SIZE.get_or_init(|| page_size());

    let new_capacity = if self.capacity == 0 {
      min_capacity.max(self.initial_capacity())
    } else {
      (self.capacity * MVEC_GROWTH_FACTOR).max(min_capacity)
    };
    let bytes_needed = new_capacity * mem::size_of::<T>();
    let total_pages_needed = (bytes_needed + ps - 1) / ps;

    let old_data = self.data;
    let new_page = Page::new(self.mapper, total_pages_needed * ps)?;
    let new_data_ptr = new_page.as_ref().as_ptr() as *mut T;

    if let Some(old_ptr) = old_data {
      unsafe {
        ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_data_ptr, self.len);
      }
    }

    if let Some(old_page) = self.backing.pop() {
      drop(old_page);
    }

    self.backing.push(new_page)?;
    self.data = Some(unsafe { NonNull::new_unchecked(new_data_ptr) });
    self.capacity = new_capacity;
    self.active_elements = new_capacity;
    Ok(())
  }

  fn shrink_active(
    &mut self,
    needed_elements: usize,
  ) -> Result<(), MappedVectorError> {
    let ps = *Self::PAGE_SIZE.get_or_init(|| page_size());
    let needed_pages = (needed_elements * mem::size_of::<T>() + ps - 1) / ps;

    if let Some(_page) = self.backing.pop() {
      let new_page = Page::new(self.mapper, needed_pages * ps)?;
      let new_data_ptr = new_page.as_ref().as_ptr() as *mut T;

      if let Some(old_ptr) = self.data {
        unsafe {
          ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_data_ptr, self.len);
        }
      }

      self.backing.push(new_page)?;
      self.data = Some(unsafe { NonNull::new_unchecked(new_data_ptr) });
      self.active_elements = needed_elements;
    }
    Ok(())
  }

  fn shrink_capacity(
    &mut self,
    new_capacity: usize,
  ) -> Result<(), MappedVectorError> {
    let ps = *Self::PAGE_SIZE.get_or_init(|| page_size());
    let pages_needed = (new_capacity * mem::size_of::<T>() + ps - 1) / ps;

    let old_data = self.data;
    let new_page = Page::new(self.mapper, pages_needed * ps)?;
    let new_data_ptr = new_page.as_ref().as_ptr() as *mut T;

    if let Some(old_ptr) = old_data {
      unsafe {
        ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_data_ptr, self.len);
      }
    }

    if let Some(old_page) = self.backing.pop() {
      drop(old_page);
    }

    self.backing.push(new_page)?;
    self.data = Some(unsafe { NonNull::new_unchecked(new_data_ptr) });
    self.capacity = new_capacity;
    self.active_elements = new_capacity;
    Ok(())
  }

  fn maybe_shrink(&mut self) -> Result<(), MappedVectorError> {
    if self.capacity == 0 || self.len == 0 {
      return Ok(());
    }

    let usage_ratio = self.len as f32 / self.active_elements as f32;
    if usage_ratio < MVEC_DECOMMIT_THRESHOLD {
      let needed_elements = (self.len * 2).max(self.initial_capacity());
      if needed_elements < self.active_elements {
        self.shrink_active(needed_elements)?;
      }
    }

    let total_usage_ratio = self.len as f32 / self.capacity as f32;
    if total_usage_ratio < MVEC_UNMAP_THRESHOLD {
      let minimal_capacity = self.len.max(self.initial_capacity());
      if minimal_capacity < self.capacity {
        self.shrink_capacity(minimal_capacity)?;
      }
    }

    Ok(())
  }

  pub fn push(&mut self, value: T) -> Result<(), MappedVectorError> {
    self.grow(self.len + 1)?;

    unsafe {
      let data_ptr = self.data.unwrap().as_ptr();
      ptr::write(data_ptr.add(self.len), value);
      self.len += 1;
    }

    Ok(())
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.len == 0 {
      return None;
    }

    unsafe {
      self.len -= 1;
      let data_ptr = self.data.unwrap().as_ptr();
      let value = ptr::read(data_ptr.add(self.len));

      self.maybe_shrink().ok(); // Ignore shrink errors for now

      Some(value)
    }
  }

  pub fn insert(
    &mut self,
    index: usize,
    value: T,
  ) -> Result<(), MappedVectorError> {
    if index > self.len {
      return Err(MappedVectorError::IndexOutOfBounds);
    }

    self.grow(self.len + 1)?;

    unsafe {
      let data_ptr = self.data.unwrap().as_ptr();
      ptr::copy(
        data_ptr.add(index),
        data_ptr.add(index + 1),
        self.len - index,
      );
      ptr::write(data_ptr.add(index), value);
      self.len += 1;
    }

    Ok(())
  }

  pub fn remove(&mut self, index: usize) -> Option<T> {
    if index >= self.len {
      return None;
    }

    unsafe {
      let data_ptr = self.data.unwrap().as_ptr();
      let value = ptr::read(data_ptr.add(index));
      ptr::copy(
        data_ptr.add(index + 1),
        data_ptr.add(index),
        self.len - index - 1,
      );
      self.len -= 1;

      self.maybe_shrink().ok();

      Some(value)
    }
  }

  pub fn clear(&mut self) {
    unsafe {
      if let Some(data_ptr) = self.data {
        for i in 0..self.len {
          ptr::drop_in_place(data_ptr.as_ptr().add(i));
        }
      }
    }
    self.len = 0;
    self.maybe_shrink().ok();
  }

  pub fn reserve(
    &mut self,
    additional: usize,
  ) -> Result<(), MappedVectorError> {
    self.grow(self.len + additional)
  }

  pub fn get(&self, index: usize) -> Option<&T> {
    if index >= self.len {
      return None;
    }

    unsafe {
      let data_ptr = self.data?.as_ptr();
      Some(&*data_ptr.add(index))
    }
  }

  pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
    if index >= self.len {
      return None;
    }

    unsafe {
      let data_ptr = self.data?.as_ptr();
      Some(&mut *data_ptr.add(index))
    }
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn capacity(&self) -> usize {
    self.capacity
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tinyalloc_sys::vm::Mapper;

  #[cfg(unix)]
  use tinyalloc_sys::system::posix::PosixMapper;
  #[cfg(windows)]
  use tinyalloc_sys::system::windows::WindowsMapper;

  #[cfg(unix)]
  static BACKING_MAPPER: PosixMapper = PosixMapper;
  #[cfg(windows)]
  static BACKING_MAPPER: WindowsMapper = WindowsMapper;

  static MAPPER: &dyn Mapper = &BACKING_MAPPER;

  #[test]
  fn test_new() {
    let mvec: MappedVector<i32> = MappedVector::new(MAPPER);
    assert_eq!(mvec.len(), 0);
    assert_eq!(mvec.capacity(), 0);
    assert!(mvec.is_empty());
  }

  #[test]
  fn test_push_pop() {
    let mut mvec: MappedVector<i32> = MappedVector::new(MAPPER);

    mvec.push(1).unwrap();
    mvec.push(2).unwrap();

    assert_eq!(mvec.len(), 2);
    assert_eq!(mvec.pop(), Some(2));
    assert_eq!(mvec.pop(), Some(1));
    assert_eq!(mvec.pop(), None);
  }

  #[test]
  fn test_get() {
    let mut mvec: MappedVector<i32> = MappedVector::new(MAPPER);

    mvec.push(10).unwrap();
    mvec.push(20).unwrap();

    assert_eq!(mvec.get(0), Some(&10));
    assert_eq!(mvec.get(1), Some(&20));
    assert_eq!(mvec.get(2), None);
  }

  #[test]
  fn test_insert_remove() {
    let mut mvec: MappedVector<i32> = MappedVector::new(MAPPER);

    mvec.push(1).unwrap();
    mvec.push(3).unwrap();
    mvec.insert(1, 2).unwrap();

    assert_eq!(mvec.len(), 3);
    assert_eq!(mvec.get(0), Some(&1));
    assert_eq!(mvec.get(1), Some(&2));
    assert_eq!(mvec.get(2), Some(&3));

    assert_eq!(mvec.remove(1), Some(2));
    assert_eq!(mvec.len(), 2);
  }

  #[test]
  fn test_generic_preallocation() {
    let mvec: MappedVector<i32, 20> = MappedVector::new(MAPPER);
    assert!(mvec.initial_capacity() >= 20);
  }
}
