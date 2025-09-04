use std::{
  mem::MaybeUninit,
  ptr::NonNull,
};

use tinyalloc_sys::{
  page::Page,
  vm::Mapper,
};

// Philosophy: syscalls are DAMN slow so DON'T ALLOCATE ANYTHING UNTIL WE ACTUALLY APPEND TO THE VECTOR
// Resizes are bigger in order to prevent often resizes

pub struct MappedVector<'mapper, T> {
  data: NonNull<T>,
  backing: Page<'mapper>,
}

impl<'mapper, T> MappedVector<'mapper, T> {
  pub fn new(mapper: &'mapper dyn Mapper) -> Self {
    let mut page = Page::new(mapper, 0).unwrap();
    let data = page.as_mut().as_mut_ptr() as *mut T;
    Self {
      data: NonNull::new(data).unwrap(),
      backing: page,
    }
  }
}
