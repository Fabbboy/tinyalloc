use core::{
  mem::MaybeUninit,
  ops::{
    Deref,
    DerefMut,
  },
  slice,
};

#[derive(Debug)]
pub enum ArrayError {
  OutOfBounds { index: usize, size: usize },
  InsufficientCapacity { have: usize, need: usize },
}

#[derive(Debug)]
pub struct Array<T, const SIZE: usize> {
  data: [MaybeUninit<T>; SIZE],
  len: usize,
}

impl<T, const SIZE: usize> Array<T, SIZE> {
  pub const fn new() -> Self {
    Self {
      data: unsafe { MaybeUninit::uninit().assume_init() },
      len: 0,
    }
  }

  pub const fn len(&self) -> usize {
    self.len
  }

  pub const fn capacity(&self) -> usize {
    SIZE
  }

  pub const fn is_empty(&self) -> bool {
    self.len == 0
  }

  pub const fn is_full(&self) -> bool {
    self.len == SIZE
  }

  pub fn push(&mut self, value: T) -> Result<(), ArrayError> {
    if self.is_full() {
      return Err(ArrayError::InsufficientCapacity {
        have: SIZE,
        need: SIZE + 1,
      });
    }

    unsafe {
      self.data.get_unchecked_mut(self.len).write(value);
    }
    self.len += 1;
    Ok(())
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.is_empty() {
      None
    } else {
      self.len -= 1;
      Some(unsafe { self.data.get_unchecked(self.len).assume_init_read() })
    }
  }

  pub fn get(&self, index: usize) -> Result<&T, ArrayError> {
    if index >= self.len {
      Err(ArrayError::OutOfBounds {
        index,
        size: self.len,
      })
    } else {
      Ok(unsafe { self.data.get_unchecked(index).assume_init_ref() })
    }
  }

  pub fn get_mut(&mut self, index: usize) -> Result<&mut T, ArrayError> {
    if index >= self.len {
      Err(ArrayError::OutOfBounds {
        index,
        size: self.len,
      })
    } else {
      Ok(unsafe { self.data.get_unchecked_mut(index).assume_init_mut() })
    }
  }

  pub unsafe fn get_unchecked(&self, index: usize) -> &T {
    unsafe { self.data.get_unchecked(index).assume_init_ref() }
  }

  pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
    unsafe { self.data.get_unchecked_mut(index).assume_init_mut() }
  }

  pub fn clear(&mut self) {
    while self.pop().is_some() {}
  }

  pub fn as_slice(&self) -> &[T] {
    unsafe { slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
  }

  pub fn as_mut_slice(&mut self) -> &mut [T] {
    unsafe {
      slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len)
    }
  }
}

impl<T, const SIZE: usize> Default for Array<T, SIZE> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const SIZE: usize> Deref for Array<T, SIZE> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<T, const SIZE: usize> DerefMut for Array<T, SIZE> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut_slice()
  }
}

impl<T, const SIZE: usize> Drop for Array<T, SIZE> {
  fn drop(&mut self) {
    self.clear();
  }
}

#[cfg(test)]
mod tests;
