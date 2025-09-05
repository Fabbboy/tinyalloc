use std::ptr::NonNull;

use enumset::{
  EnumSet,
  EnumSetType,
};
use getset::Getters;

use crate::vm::{
  MapError,
  Mapper,
};

#[derive(EnumSetType, Debug)]
pub enum PageFlag {
  Mapped,
  Committed,
  Protected,
}

#[derive(Getters)]
pub struct Page<'mapper> {
  #[getset(get = "pub")]
  mapper: &'mapper dyn Mapper,
  #[getset(get = "pub")]
  ptr: NonNull<[u8]>,
  #[getset(get = "pub")]
  flags: EnumSet<PageFlag>,
}

impl<'mapper> Page<'mapper> {
  pub fn new(
    mapper: &'mapper dyn Mapper,
    size: usize,
    committed: bool,
  ) -> Result<Self, MapError> {
    let ptr = mapper.map(size, committed)?;
    Ok(Self {
      mapper,
      ptr,
      flags: PageFlag::Mapped | PageFlag::Committed,
    })
  }

  pub fn commit(&mut self) -> Result<(), MapError> {
    self.mapper.commit(self.ptr)?;
    self.flags |= PageFlag::Committed;
    self.flags -= PageFlag::Protected;
    Ok(())
  }

  pub fn decommit(&mut self) -> Result<(), MapError> {
    self.mapper.decommit(self.ptr)?;
    self.flags -= PageFlag::Committed;
    Ok(())
  }

  pub fn protect(&mut self) -> Result<(), MapError> {
    self.mapper.protect(self.ptr)?;
    self.flags |= PageFlag::Protected;
    Ok(())
  }

  pub fn is_mapped(&self) -> bool {
    self.flags.contains(PageFlag::Mapped)
  }

  pub fn is_committed(&self) -> bool {
    self.flags.contains(PageFlag::Committed)
  }

  pub fn is_protected(&self) -> bool {
    self.flags.contains(PageFlag::Protected)
  }
}

impl<'mapper> Drop for Page<'mapper> {
  fn drop(&mut self) {
    if self.is_mapped() {
      self.mapper.unmap(self.ptr);
    }
  }
}

impl<'mapper> Page<'mapper> {
  pub fn as_slice(&self) -> Option<&[u8]> {
    if self.is_committed() && !self.is_protected() {
      // Safety: we verify that the page is both committed and unprotected
      // before exposing the backing memory.
      Some(unsafe { self.ptr.as_ref() })
    } else {
      None
    }
  }

  pub fn as_slice_mut(&mut self) -> Option<&mut [u8]> {
    if self.is_committed() && !self.is_protected() {
      // Safety: we verify that the page is both committed and unprotected
      // before exposing the backing memory mutably.
      Some(unsafe { self.ptr.as_mut() })
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::size::page_size;

  #[cfg(unix)]
  use crate::system::posix::PosixMapper;
  #[cfg(windows)]
  use crate::system::windows::WindowsMapper;

  #[cfg(unix)]
  static BACKING_MAPPER: PosixMapper = PosixMapper;
  #[cfg(windows)]
  static BACKING_MAPPER: WindowsMapper = WindowsMapper;

  static MAPPER: &dyn Mapper = &BACKING_MAPPER;

  #[test]
  fn test_page_raii_basic() {
    let page = Page::new(MAPPER, page_size(), true);
    assert!(page.is_ok());
    assert!(page.unwrap().is_mapped());
  }

  #[test]
  fn test_page_raii_operations() {
    let mut page = Page::new(MAPPER, page_size(), true).unwrap();

    assert!(page.is_mapped());

    page.decommit().unwrap();
    assert!(!page.is_committed());

    page.commit().unwrap();
    assert!(page.is_committed());

    page.protect().unwrap();
    assert!(page.is_protected());
  }

  #[test]
  fn test_page_raii_write_after_commit() {
    let mut page = Page::new(MAPPER, page_size(), true).unwrap();

    page.decommit().unwrap();
    assert!(!page.is_committed());

    page.commit().unwrap();
    assert!(page.is_committed());

    if let Some(slice) = page.as_slice_mut() {
      slice[0] = 42;
    }
    if let Some(slice) = page.as_slice() {
      assert_eq!(slice[0], 42);
    }
  }
}
