use std::{
  ptr,
  ptr::NonNull,
};
use tinyalloc_core::{
  size::page_align,
  vm::{
    MapError,
    Mapper,
  },
};

mod mapper {
  use std::{
    ptr::NonNull,
    slice,
  };

  #[cfg(all(unix, not(target_os = "macos")))]
  use libc::MADV_DONTNEED;
  #[cfg(all(unix, target_os = "macos"))]
  use libc::{
    MADV_FREE,
    MAP_ANON,
  };
  use libc::{
    MAP_ANONYMOUS,
    MAP_PRIVATE,
    PROT_READ,
    PROT_WRITE,
  };

  pub static PERMISSIONS: i32 = PROT_READ | PROT_WRITE;
  #[cfg(all(unix, not(target_os = "macos")))]
  pub static FLAGS: i32 = MAP_ANONYMOUS | MAP_PRIVATE;
  #[cfg(all(unix, target_os = "macos"))]
  pub static FLAGS: i32 = MAP_ANON | MAP_PRIVATE;

  #[cfg(all(unix, not(target_os = "macos")))]
  pub static UNINTERESTED: i32 = MADV_DONTNEED;
  #[cfg(all(unix, target_os = "macos"))]
  pub static UNINTERESTED: i32 = MADV_FREE;

  pub static TRASH_FD: i32 = -1;

  pub fn check_map(result: *mut libc::c_void) -> Result<super::NonNull<[u8]>, super::MapError> {
    if result == libc::MAP_FAILED {
      Err(super::MapError)
    } else {
      let slice = unsafe { slice::from_raw_parts_mut(result as *mut u8, 0) };
      Ok(unsafe { NonNull::new_unchecked(slice) })
    }
  }

  pub const fn cptr(slice: &NonNull<[u8]>) -> *mut libc::c_void {
    unsafe { slice.as_ref().as_ptr() as *mut libc::c_void }
  }
}

pub struct PosixMapper;

#[cfg(unix)]
impl Mapper for PosixMapper {
  fn map(&self, size: usize) -> Result<NonNull<[u8]>, MapError> {
    let size = page_align(size);
    let result = unsafe {
      libc::mmap(
        ptr::null_mut(),
        size,
        mapper::PERMISSIONS,
        mapper::FLAGS,
        mapper::TRASH_FD,
        0,
      )
    };

    mapper::check_map(result)
  }
  fn unmap(&self, ptr: NonNull<[u8]>) {
    let _ = unsafe { libc::munmap(mapper::cptr(&ptr), ptr.len()) };
  }
  fn commit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
  fn decommit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
  fn protect(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
  fn unprotect(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    _ = ptr;
    return Err(MapError);
  }
}
