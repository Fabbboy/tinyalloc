#[cfg(windows)]
use std::{
  ffi::c_void,
  ptr,
  ptr::NonNull,
  slice,
};

use crate::vm::Mapper;
#[cfg(windows)]
use crate::{
  size::page_align,
  vm::MapError,
};

#[cfg(windows)]
mod inner {
  pub use windows_sys::Win32::System::Memory::{
    MEM_COMMIT,
    MEM_DECOMMIT,
    MEM_RELEASE,
    MEM_RESERVE,
    PAGE_NOACCESS,
    PAGE_READWRITE,
  };

  pub const MEM_RESERVE_COMMIT: u32 = MEM_RESERVE | MEM_COMMIT;
  pub const PAGE_RW: u32 = PAGE_READWRITE;
  pub const PAGE_NONE: u32 = PAGE_NOACCESS;
  pub const MEM_DECOMMIT_FLAG: u32 = MEM_DECOMMIT;
  pub const MEM_RELEASE_FLAG: u32 = MEM_RELEASE;
}

pub struct WindowsMapper;

#[cfg(not(windows))]
impl Mapper for WindowsMapper {}

#[cfg(windows)]
impl WindowsMapper {
  fn check_result(&self, result: *mut c_void) -> Result<*mut c_void, MapError> {
    if result.is_null() {
      Err(MapError)
    } else {
      Ok(result)
    }
  }

  fn check_bool(&self, result: i32) -> Result<(), MapError> {
    if result == 0 { Err(MapError) } else { Ok(()) }
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
  fn map(
    &self,
    size: usize,
    committed: bool,
  ) -> Result<NonNull<[u8]>, MapError> {
    let size = page_align(size);

    let permission = if committed {
      inner::PAGE_RW
    } else {
      inner::PAGE_NONE
    };

    let result = unsafe {
      VirtualAlloc(ptr::null_mut(), size, inner::MEM_RESERVE_COMMIT, permission)
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

  fn commit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    let result = unsafe {
      VirtualAlloc(
        ptr.as_ptr() as *mut c_void,
        ptr.len(),
        inner::MEM_COMMIT,
        inner::PAGE_RW,
      )
    };
    self.check_result(result)?;
    Ok(())
  }

  fn decommit(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    let result = unsafe {
      VirtualFree(
        ptr.as_ptr() as *mut c_void,
        ptr.len(),
        inner::MEM_DECOMMIT_FLAG,
      )
    };
    self.check_bool(result)
  }

  fn protect(&self, ptr: NonNull<[u8]>) -> Result<(), MapError> {
    let mut old_protect = 0;
    let result = unsafe {
      VirtualProtect(
        ptr.as_ptr() as *mut c_void,
        ptr.len(),
        inner::PAGE_NONE,
        &mut old_protect,
      )
    };
    self.check_bool(result)
  }
}
