use libc::{
  MAP_PRIVATE,
  PROT_NONE,
  PROT_READ,
  PROT_WRITE,
};
use std::{
  ptr,
  ptr::NonNull,
  slice,
};
use tinyalloc_core::{
  size::page_align,
  vm::{
    MapError,
    Mapper,
  },
};

pub struct PosixMapper;

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

    check_map(result, size)
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
