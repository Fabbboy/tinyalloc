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
  /// "Active" elements concept kept for policy thresholds; with a single page
  /// we remap instead of partial commit, so this equals `capacity` after changes.
  active_elements: usize,
  /// Single backing mapping
  backing: Option<Page<'mapper>>,
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
      backing: None,
      mapper,
    }
  }

  #[inline]
  fn elements_per_page(ps: usize) -> usize {
    let elem = mem::size_of::<T>().max(1);
    ps / elem
  }

  fn initial_capacity(&self) -> usize {
    let ps = *Self::PAGE_SIZE.get_or_init(page_size);
    N * Self::elements_per_page(ps)
  }

  /// Ensure capacity for at least `min_capacity` elements.
  /// With a single page, this remaps to a new page if needed and copies old data.
  fn grow(&mut self, min_capacity: usize) -> Result<(), MappedVectorError> {
    if self.capacity >= min_capacity {
      // If someone decommitted (conceptually), "commit" back if API supports it.
      if let Some(page) = self.backing.as_mut() {
        if !page.is_committed() {
          page.commit()?;
        }
      }
      return Ok(());
    }

    let ps = *Self::PAGE_SIZE.get_or_init(page_size);

    let new_capacity = if self.capacity == 0 {
      min_capacity.max(self.initial_capacity())
    } else {
      (self.capacity * MVEC_GROWTH_FACTOR).max(min_capacity)
    };

    let bytes_needed = new_capacity
      .checked_mul(mem::size_of::<T>())
      .expect("capacity overflow");
    let total_bytes = ((bytes_needed + ps - 1) / ps) * ps;

    // Map new single region
    let mut new_page = Page::new(self.mapper, total_bytes)?;
    let new_data_ptr = new_page.as_mut().as_mut_ptr() as *mut T;

    // Copy old payload before dropping old page
    if let Some(old_ptr) = self.data {
      unsafe {
        ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_data_ptr, self.len);
      }
    }

    // Drop old page last
    if let Some(old_page) = self.backing.take() {
      drop(old_page);
    }

    self.backing = Some(new_page);
    self.data = Some(unsafe { NonNull::new_unchecked(new_data_ptr) });
    self.capacity = new_capacity;
    self.active_elements = new_capacity;
    Ok(())
  }

  /// "Decommit" policy: with a single page we just remap smaller and copy.
  fn shrink_active(
    &mut self,
    needed_elements: usize,
  ) -> Result<(), MappedVectorError> {
    self.remap_to_capacity(needed_elements)
  }

  /// Hard shrink of total capacity; same remap in single-page world.
  fn shrink_capacity(
    &mut self,
    new_capacity: usize,
  ) -> Result<(), MappedVectorError> {
    self.remap_to_capacity(new_capacity)
  }

  /// Remap the single page to fit `new_capacity` elements and copy current data.
  fn remap_to_capacity(
    &mut self,
    new_capacity: usize,
  ) -> Result<(), MappedVectorError> {
    let ps = *Self::PAGE_SIZE.get_or_init(page_size);
    let bytes_needed = new_capacity
      .checked_mul(mem::size_of::<T>())
      .expect("capacity overflow");
    let total_bytes = if bytes_needed == 0 {
      ps
    } else {
      ((bytes_needed + ps - 1) / ps) * ps
    };

    let mut new_page = Page::new(self.mapper, total_bytes)?;
    let new_data_ptr = new_page.as_mut().as_mut_ptr() as *mut T;

    if let Some(old_ptr) = self.data {
      unsafe {
        ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_data_ptr, self.len);
      }
    }

    if let Some(old_page) = self.backing.take() {
      drop(old_page);
    }

    self.backing = Some(new_page);
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
    }
    self.len += 1;
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
      let _ = self.maybe_shrink();
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
    }
    self.len += 1;
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
      let _ = self.maybe_shrink();
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
    let _ = self.maybe_shrink();
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

  #[inline]
  pub fn len(&self) -> usize {
    self.len
  }

  #[inline]
  pub fn capacity(&self) -> usize {
    self.capacity
  }

  #[inline]
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

  #[test]
  fn test_clear_and_repush_behavior() {
    let mut mvec: MappedVector<i32, 1> = MappedVector::new(MAPPER);

    for i in 0..10000 {
      mvec.push(i).unwrap();
    }
    for _ in 0..10000 {
      mvec.pop();
    }
    assert_eq!(mvec.len(), 0);

    for i in 0..10000 {
      mvec.push(i).unwrap();
    }
  }

  #[test]
  fn test_large_objects_push_pop_push() {
    #[repr(align(4096))]
    struct LargeObject {
      _data: [u8; 4096],
    }

    let mut mvec: MappedVector<LargeObject, 1> = MappedVector::new(MAPPER);

    for i in 0..20000 {
      let obj = LargeObject {
        _data: [i as u8; 4096],
      };
      mvec.push(obj).unwrap();
    }

    assert_eq!(mvec.len(), 20000);

    for _ in 0..20000 {
      mvec.pop().unwrap();
    }

    assert_eq!(mvec.len(), 0);

    let start = std::time::Instant::now();
    for i in 0..20000 {
      let obj = LargeObject {
        _data: [i as u8; 4096],
      };
      mvec.push(obj).unwrap();
    }
    let duration = start.elapsed();

    assert_eq!(mvec.len(), 20000);
    assert!(
      duration.as_secs() < 5,
      "Push operation took too long: {:?}",
      duration
    );
  }
}
