#[cfg(unix)]
use std::{
  ptr,
  ptr::NonNull,
  slice,
};

use tinyalloc_core::vm::Mapper;
#[cfg(unix)]
use tinyalloc_core::{
  size::page_align,
  vm::MapError,
};

#[cfg(unix)]
mod inner {
  use libc::{
    MAP_PRIVATE,
    PROT_NONE,
    PROT_READ,
    PROT_WRITE,
  };

  #[cfg(not(target_os = "macos"))]
  use libc::{
    MADV_DONTNEED,
    MAP_ANONYMOUS,
  };

  #[cfg(target_os = "macos")]
  use libc::{
    MADV_FREE,
    MAP_ANON,
  };

  #[cfg(not(target_os = "macos"))]
  pub const MAP_FLAGS: i32 = MAP_PRIVATE | MAP_ANONYMOUS;
  #[cfg(target_os = "macos")]
  pub const MAP_FLAGS: i32 = MAP_PRIVATE | MAP_ANON;

  #[cfg(not(target_os = "macos"))]
  pub const DECOMMIT_FLAG: i32 = MADV_DONTNEED;
  #[cfg(target_os = "macos")]
  pub const DECOMMIT_FLAG: i32 = MADV_FREE;

  pub const PERMISSIONS_RW: i32 = PROT_READ | PROT_WRITE;
  pub const PERMISSIONS_NONE: i32 = PROT_NONE;
  pub const TRASH_FD: i32 = -1;
}

pub struct PosixMapper;

#[cfg(not(unix))]
impl Mapper for PosixMapper {}

#[cfg(unix)]
impl PosixMapper {
  fn check_syscall(&self, result: libc::c_int) -> Result<(), MapError> {
    if result == 0 { Ok(()) } else { Err(MapError) }
  }

  fn cptr(&self, ptr: NonNull<[u8]>) -> *mut libc::c_void {
    ptr.as_ptr() as *mut libc::c_void
  }
}

#[cfg(unix)]
impl Mapper for PosixMapper {
  fn map(&self, size: usize) -> Result<NonNull<[u8]>, MapError> {
    let size = page_align(size);
    let result = unsafe {
      libc::mmap(
        ptr::null_mut(),
        size,
        inner::PERMISSIONS_RW,
        inner::MAP_FLAGS,
        inner::TRASH_FD,
        0,
      )
    };

    if result == libc::MAP_FAILED {
      return Err(MapError);
    }

    let slice = unsafe { slice::from_raw_parts_mut(result as *mut u8, size) };
    Ok(NonNull::new(slice).unwrap())
  }

  fn unmap(&self, ptr: NonNull<[u8]>) {
    unsafe { libc::munmap(self.cptr(ptr), ptr.len()) };
  }

  fn commit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    let result = unsafe { libc::mprotect(self.cptr(ptr), ptr.len(), inner::PERMISSIONS_RW) };
    self.check_syscall(result)
  }

  fn decommit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    let result = unsafe { libc::madvise(self.cptr(ptr), ptr.len(), inner::DECOMMIT_FLAG) };
    self.check_syscall(result)
  }

  fn protect(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    let result = unsafe { libc::mprotect(self.cptr(ptr), ptr.len(), inner::PERMISSIONS_NONE) };
    self.check_syscall(result)
  }
}

