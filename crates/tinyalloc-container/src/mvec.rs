use std::{
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

use crate::queue::Queue;

pub const MVEC_GROWTH_FACTOR: usize = 2;
pub const MVEC_DECOMMIT_THRESHOLD: f32 = 0.25;
pub const MVEC_UNMAP_THRESHOLD: f32 = 0.125;

pub struct MappedVector<'mapper, T> {
  data: Option<NonNull<T>>,
  len: usize,
  capacity: usize,
  backing: Queue<'mapper, Page<'mapper>>,
  mapper: &'mapper dyn Mapper,
}

impl<'mapper, T> MappedVector<'mapper, T> {
  pub fn new(mapper: &'mapper dyn Mapper) -> Self {
    Self {
      data: None,
      len: 0,
      capacity: 0,
      backing: Queue::new(mapper),
      mapper,
    }
  }

  fn elements_per_page(&self) -> usize {
    page_size() / mem::size_of::<T>()
  }

  fn ensure_capacity(&mut self, min_capacity: usize) -> Result<(), MapError> {
    if self.capacity >= min_capacity {
      return Ok(());
    }

    let new_capacity = if self.capacity == 0 {
      min_capacity.max(self.elements_per_page())
    } else {
      (self.capacity * MVEC_GROWTH_FACTOR).max(min_capacity)
    };

    let pages_needed =
      (new_capacity * mem::size_of::<T>() + page_size() - 1) / page_size();

    for _ in 0..pages_needed {
      let page = Page::new(self.mapper, page_size())?;
      self.backing.push(page)?;
    }

    if self.data.is_none() {
      let first_page = Page::new(self.mapper, page_size())?;
      let data_ptr = first_page.as_ref().as_ptr() as *mut T;
      self.data = Some(unsafe { NonNull::new_unchecked(data_ptr) });
      self.backing.push(first_page)?;
    }

    self.capacity = new_capacity;
    Ok(())
  }

  pub fn push(&mut self, value: T) -> Result<(), MapError> {
    self.ensure_capacity(self.len + 1)?;

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
      Some(std::ptr::read(data_ptr.add(self.len)))
    }
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
    let mut mvec = MappedVector::new(MAPPER);

    mvec.push(1).unwrap();
    mvec.push(2).unwrap();

    assert_eq!(mvec.len(), 2);
    assert_eq!(mvec.pop(), Some(2));
    assert_eq!(mvec.pop(), Some(1));
    assert_eq!(mvec.pop(), None);
  }

  #[test]
  fn test_get() {
    let mut mvec = MappedVector::new(MAPPER);

    mvec.push(10).unwrap();
    mvec.push(20).unwrap();

    assert_eq!(mvec.get(0), Some(&10));
    assert_eq!(mvec.get(1), Some(&20));
    assert_eq!(mvec.get(2), None);
  }
}
