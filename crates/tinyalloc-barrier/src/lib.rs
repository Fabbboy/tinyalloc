use std::{
  cell::UnsafeCell,
  thread::{
    self,
    ThreadId,
  },
};

pub struct Barrier<T> {
  inner: UnsafeCell<T>,
  thread: ThreadId,
}

impl<T> Barrier<T> {
  fn this() -> ThreadId {
    thread::current().id()
  }

  pub fn new(value: T) -> Self {
    Self {
      inner: UnsafeCell::new(value),
      thread: Self::this(),
    }
  }

  pub fn get(&self) -> Option<&T> {
    if self.thread == Self::this() {
      Some(unsafe { &*self.inner.get() })
    } else {
      None
    }
  }

  pub fn get_mut(&self) -> Option<&mut T> {
    if self.thread == Self::this() {
      Some(unsafe { &mut *self.inner.get() })
    } else {
      None
    }
  }

  pub unsafe fn release(self) -> Option<T> {
    if self.thread == Self::this() {
      Some(self.inner.into_inner())
    } else {
      None
    }
  }
}

unsafe impl<T> Sync for Barrier<T> where T: Send {}
unsafe impl<T> Send for Barrier<T> where T: Send {}

#[cfg(test)]
mod tests;