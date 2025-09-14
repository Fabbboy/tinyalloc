#[cfg(windows)]
use std::{
  ffi::c_void,
  num::NonZeroUsize,
  ptr,
  ptr::NonNull,
  slice,
};

#[cfg(windows)]
use enumset::EnumSet;

use crate::mapper::{
  Mapper,
  MapperRequires,
  Protection,
};

#[cfg(windows)]
use crate::{
  MapError,
  size::page_align,
};

#[cfg(windows)]
mod inner {
  pub use windows_sys::Win32::System::Memory::{
    MEM_COMMIT,
    MEM_DECOMMIT,
    MEM_RELEASE,
    MEM_RESERVE,
    MEM_RESET,
    PAGE_NOACCESS,
    PAGE_READONLY,
    PAGE_READWRITE,
  };

  pub const MEM_RESERVE_COMMIT: u32 = MEM_RESERVE | MEM_COMMIT;
  pub const PAGE_RW: u32 = PAGE_READWRITE;
  pub const PAGE_R: u32 = PAGE_READONLY;
  pub const PAGE_NONE: u32 = PAGE_NOACCESS;
  pub const MEM_DECOMMIT_FLAG: u32 = MEM_DECOMMIT;
  pub const MEM_RELEASE_FLAG: u32 = MEM_RELEASE;
  pub const MEM_RESET_FLAG: u32 = MEM_RESET;
}

#[derive(Clone, Debug)]
pub struct WindowsMapper;

impl MapperRequires for WindowsMapper {}

#[cfg(not(windows))]
impl Mapper for WindowsMapper {}

#[cfg(windows)]
impl WindowsMapper {
  fn check_result(&self, result: *mut c_void) -> Result<*mut c_void, MapError> {
    if result.is_null() {
      Err(MapError::OutOfMemory)
    } else {
      Ok(result)
    }
  }

  fn check_bool(&self, result: i32) -> Result<(), MapError> {
    if result == 0 {
      Err(MapError::ProtectFailed)
    } else {
      Ok(())
    }
  }

  fn to_page_protection(prot: EnumSet<Protection>) -> u32 {
    let rf = prot.contains(Protection::Read);
    let wf = prot.contains(Protection::Write);
    match (rf, wf) {
      (true, true) => inner::PAGE_RW,
      (true, false) => inner::PAGE_R,
      (false, true) => inner::PAGE_RW, // Windows doesn't have write-only
      (false, false) => inner::PAGE_NONE,
    }
  }
}

#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
  VirtualAlloc,
  VirtualFree,
  VirtualProtect,
};

#[cfg(windows)]
impl Mapper for WindowsMapper {
  fn map(&self, size: NonZeroUsize) -> Result<NonNull<[u8]>, MapError> {
    let size = page_align(size.get());

    // Always use MEM_RESERVE_COMMIT with PAGE_NONE initially
    // This matches the pattern of reserving and committing in one step
    let result = unsafe {
      VirtualAlloc(
        ptr::null_mut(),
        size,
        inner::MEM_RESERVE_COMMIT,
        inner::PAGE_NONE,
      )
    };

    let ptr = self.check_result(result)?;
    let slice = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, size) };
    Ok(NonNull::new(slice).unwrap())
  }

  fn unmap(&self, ptr: NonNull<[u8]>) {
    unsafe {
      VirtualFree(ptr.as_ptr() as *mut c_void, 0, inner::MEM_RELEASE_FLAG);
    }
  }

  fn decommit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    // On Windows, we use VirtualFree with MEM_DECOMMIT to decommit pages
    let result = unsafe {
      VirtualFree(
        ptr.as_ptr() as *mut c_void,
        ptr.len(),
        inner::MEM_DECOMMIT_FLAG,
      )
    };
    self.check_bool(result)
  }

  fn protect(
    &self,
    ptr: NonNull<[u8]>,
    prot: EnumSet<Protection>,
  ) -> Result<(), MapError> {
    let prot_flags = Self::to_page_protection(prot);
    let mut old_protect = 0;

    let result = unsafe {
      VirtualProtect(
        ptr.as_ptr() as *mut c_void,
        ptr.len(),
        prot_flags,
        &mut old_protect,
      )
    };
    self.check_bool(result)
  }
}

#[cfg(all(test, windows))]
mod tests {
  use std::num::NonZero;

  use enumset::EnumSet;

  use super::*;
  use crate::{
    GLOBAL_MAPPER,
    mapper::Protection,
  };

  #[test]
  fn test_to_prot_all_combinations() {
    assert_eq!(
      WindowsMapper::to_page_protection(EnumSet::empty()),
      inner::PAGE_NONE
    );
    assert_eq!(
      WindowsMapper::to_page_protection(Protection::Read.into()),
      inner::PAGE_R
    );
    assert_eq!(
      WindowsMapper::to_page_protection(Protection::Write.into()),
      inner::PAGE_RW
    );
    assert_eq!(
      WindowsMapper::to_page_protection(EnumSet::all()),
      inner::PAGE_RW
    );
  }

  #[test]
  fn test_map_and_unmap() {
    let result = GLOBAL_MAPPER.map(NonZero::new(4096).unwrap());
    assert!(result.is_ok());

    let ptr = result.unwrap();
    assert!(ptr.len() >= 4096);

    GLOBAL_MAPPER.unmap(ptr);
  }

  #[test]
  fn test_protect_with_different_permissions() {
    let ptr = GLOBAL_MAPPER.map(NonZero::new(4096).unwrap()).unwrap();

    assert!(GLOBAL_MAPPER.protect(ptr, EnumSet::empty()).is_ok());
    assert!(GLOBAL_MAPPER.protect(ptr, Protection::Read.into()).is_ok());
    assert!(GLOBAL_MAPPER.protect(ptr, Protection::Write.into()).is_ok());
    assert!(GLOBAL_MAPPER.protect(ptr, EnumSet::all()).is_ok());

    GLOBAL_MAPPER.unmap(ptr);
  }

  #[test]
  fn test_decommit() {
    let ptr = GLOBAL_MAPPER.map(NonZero::new(4096).unwrap()).unwrap();

    // First protect with RW so we can write to it
    GLOBAL_MAPPER.protect(ptr, EnumSet::all()).unwrap();

    let decommit_result = GLOBAL_MAPPER.decommit(ptr);
    assert!(decommit_result.is_ok());
    GLOBAL_MAPPER.unmap(ptr);
  }

  #[test]
  fn test_large_allocation() {
    let size = 1024 * 1024;
    let ptr = GLOBAL_MAPPER.map(NonZero::new(size).unwrap()).unwrap();
    assert!(ptr.len() >= size);

    GLOBAL_MAPPER.protect(ptr, EnumSet::all()).unwrap();

    GLOBAL_MAPPER.unmap(ptr);
  }
}
