use std::ptr::NonNull;

use getset::{Getters, MutGetters};
use tinyalloc_sys::{page::Page, vm::{MapError, Mapper}};

#[derive(Getters, MutGetters)]
pub struct Arena<'mapper> {
  page: Page<'mapper>,
  #[getset(get = "pub", get_mut = "pub")]
  next: Option<NonNull<Arena<'mapper>>>,
}

impl<'mapper> Arena<'mapper> {
  pub fn new(system: &'mapper dyn Mapper, size: usize) -> Result<NonNull<Self>, MapError> {
    let mut page = Page::new(system, size)?;
    let slice = page.as_mut();
    let ptr = slice.as_mut_ptr() as *mut Self;
    unsafe {
      ptr.write(Self { page, next: None });
      Ok(NonNull::new_unchecked(ptr))
    }
  }
}

impl<'mapper> AsRef<[u8]> for Arena<'mapper> {
  fn as_ref(&self) -> &[u8] {
    self.page.as_ref()
  }
}

impl<'mapper> AsMut<[u8]> for Arena<'mapper> {
  fn as_mut(&mut self) -> &mut [u8] {
    self.page.as_mut()
  }
}

impl<'mapper> Drop for Arena<'mapper> {
  fn drop(&mut self) {
     self.next = None;
  }
}