use tinyalloc_alloc::config::{
  MAX_ALIGN,
  MIN_ALIGN,
  align_up,
};

use crate::TinyAlloc;
use core::ffi::{
  c_int,
  c_void,
};
use std::{
  alloc::{
    GlobalAlloc,
    Layout,
  },
  convert::TryFrom,
  mem,
  ptr,
};

const METADATA_CANARY: u32 = 0xDEADBEEF;
const TRAILER_CANARY: u32 = 0xBEEFDEAD;

static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;
const ZERO_SIZE_PTR: *mut u8 = MAX_ALIGN as *mut u8;

#[repr(C)]
struct Metadata {
  ptr: *mut u8,
  canary: u32,
  layout: Layout,
  uoffset: u32,
  ualign: u32,
}

impl Metadata {
  const SELF_SIZE: usize = mem::size_of::<Metadata>();
  const ALIGN: usize = mem::align_of::<Metadata>();

  fn new(ptr: *mut u8, layout: Layout, uoffset: u32, ualign: u32) -> Self {
    Self {
      ptr,
      canary: METADATA_CANARY,
      layout,
      uoffset,
      ualign,
    }
  }

  fn validate_canary(&self) -> bool {
    self.canary == METADATA_CANARY
  }

  fn is_valid(&self) -> bool {
    self.validate_canary() && !self.ptr.is_null()
  }

  unsafe fn from_user_ptr(user_ptr: *mut u8) -> Option<&'static Self> {
    if user_ptr.is_null() {
      return None;
    }

    let user_addr = user_ptr as usize;
    if user_addr < mem::size_of::<usize>() {
      return None;
    }

    let marker_addr = user_addr - mem::size_of::<usize>();
    if marker_addr % mem::align_of::<usize>() != 0 {
      return None;
    }

    let marker_ptr = marker_addr as *const usize;
    let offset = unsafe { ptr::read(marker_ptr) as usize };
    if offset == 0 || offset > user_addr {
      return None;
    }

    let header_addr = match user_addr.checked_sub(offset) {
      Some(addr) => addr,
      None => return None,
    };

    if header_addr % mem::align_of::<Self>() != 0 {
      return None;
    }

    let metadata_ptr = header_addr as *const Self;
    let metadata = unsafe { &*metadata_ptr };

    if metadata.is_valid() {
      Some(metadata)
    } else {
      None
    }
  }
}

#[repr(C)]
struct Trailer {
  // located at aligned((usize)ptr + (layout.size - user.size))
  canary: u32,  // BEEFDEADBEEF
  uoffset: u32, // compare with metadata uoffset or use idk
}

impl Trailer {
  const SELF_SIZE: usize = mem::size_of::<Trailer>();
  const ALIGN: usize = mem::align_of::<Trailer>();

  fn new(uoffset: u32) -> Self {
    Self {
      canary: TRAILER_CANARY,
      uoffset,
    }
  }

  fn validate_canary(&self) -> bool {
    self.canary == TRAILER_CANARY
  }

  fn is_valid(&self, expected_offset: u32) -> bool {
    self.validate_canary() && self.uoffset == expected_offset
  }
}

struct Allocator;

impl Allocator {
  fn calculate_total_layout(
    size: usize,
    align: usize,
  ) -> Option<(Layout, usize)> {
    if size == 0 {
      return Some((Layout::from_size_align(MIN_ALIGN, MIN_ALIGN).ok()?, 0));
    }

    let user_align = align.max(MAX_ALIGN);
    let metadata_end = Metadata::SELF_SIZE;
    let marker_end = metadata_end.checked_add(mem::size_of::<usize>())?;

    let user_start = align_up(marker_end, user_align);
    let user_end = user_start.checked_add(size)?;

    let trailer_start = align_up(user_end, Trailer::ALIGN);

    let total_size = trailer_start.checked_add(Trailer::SELF_SIZE)?;
    let total_align = Metadata::ALIGN.max(user_align).max(Trailer::ALIGN);

    let layout = Layout::from_size_align(total_size, total_align).ok()?;
    Some((layout, user_start))
  }

  unsafe fn allocate_raw(layout: Layout, zero_init: bool) -> *mut u8 {
    unsafe {
      if zero_init {
        GLOBAL_ALLOCATOR.alloc_zeroed(layout)
      } else {
        GLOBAL_ALLOCATOR.alloc(layout)
      }
    }
  }

  unsafe fn write_metadata(
    base_ptr: *mut u8,
    layout: Layout,
    user_offset: u32,
    align: u32,
  ) {
    let metadata = Metadata::new(base_ptr, layout, user_offset, align);
    unsafe { ptr::write(base_ptr as *mut Metadata, metadata) };
  }

  unsafe fn write_offset_marker(user_ptr: *mut u8, offset: usize) {
    let offset_ptr =
      unsafe { user_ptr.sub(mem::size_of::<usize>()) } as *mut usize;
    unsafe { ptr::write(offset_ptr, offset) };
  }

  unsafe fn calculate_trailer_start(user_ptr: *mut u8, size: usize) -> *mut u8 {
    let user_end = unsafe { user_ptr.add(size) };
    let aligned_addr = align_up(user_end as usize, Trailer::ALIGN);
    aligned_addr as *mut u8
  }

  unsafe fn write_trailer(trailer_start: *mut u8, user_offset: u32) {
    let trailer = Trailer::new(user_offset);
    unsafe { ptr::write(trailer_start as *mut Trailer, trailer) };
  }

  unsafe fn allocate_with_metadata(
    size: usize,
    align: usize,
    zero_init: bool,
  ) -> *mut u8 {
    if size == 0 {
      return ZERO_SIZE_PTR;
    }

    let (total_layout, user_offset) =
      match Self::calculate_total_layout(size, align) {
        Some(result) => result,
        None => return ptr::null_mut(),
      };

    let base_ptr = unsafe { Self::allocate_raw(total_layout, zero_init) };
    if base_ptr.is_null() {
      return ptr::null_mut();
    }

    let user_ptr = unsafe { base_ptr.add(user_offset) };

    unsafe {
      let user_offset_u32 = match u32::try_from(user_offset) {
        Ok(value) => value,
        Err(_) => return ptr::null_mut(),
      };
      let align_u32 = match u32::try_from(align) {
        Ok(value) => value,
        Err(_) => return ptr::null_mut(),
      };
      Self::write_metadata(base_ptr, total_layout, user_offset_u32, align_u32);
      Self::write_offset_marker(user_ptr, user_offset);
      let trailer_start = Self::calculate_trailer_start(user_ptr, size);
      Self::write_trailer(trailer_start, user_offset_u32);
    }

    user_ptr
  }

  unsafe fn validate_and_extract_metadata(
    user_ptr: *mut u8,
  ) -> Option<&'static Metadata> {
    if user_ptr == ZERO_SIZE_PTR {
      return None;
    }

    unsafe { Metadata::from_user_ptr(user_ptr) }
  }

  fn calculate_user_size(metadata: &Metadata) -> usize {
    metadata.layout.size() - metadata.uoffset as usize - Trailer::SELF_SIZE
  }

  unsafe fn read_trailer(
    user_ptr: *mut u8,
    user_size: usize,
  ) -> &'static Trailer {
    let trailer_start =
      unsafe { Self::calculate_trailer_start(user_ptr, user_size) };
    unsafe { &*(trailer_start as *const Trailer) }
  }

  unsafe fn deallocate_raw(ptr: *mut u8, layout: Layout) {
    unsafe { GLOBAL_ALLOCATOR.dealloc(ptr, layout) };
  }

  unsafe fn deallocate_with_metadata(user_ptr: *mut u8) -> bool {
    if user_ptr == ZERO_SIZE_PTR {
      return true;
    }

    let metadata =
      match unsafe { Self::validate_and_extract_metadata(user_ptr) } {
        Some(meta) => meta,
        None => return false,
      };

    let user_size = Self::calculate_user_size(metadata);
    let trailer = unsafe { Self::read_trailer(user_ptr, user_size) };

    if !trailer.is_valid(metadata.uoffset) {
      return false;
    }

    unsafe { Self::deallocate_raw(metadata.ptr, metadata.layout) };
    true
  }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
  let ptr =
    unsafe { Allocator::allocate_with_metadata(size, MIN_ALIGN, false) };
  ptr as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
  let total_size = match nmemb.checked_mul(size) {
    Some(total) => total,
    None => return ptr::null_mut(),
  };

  let ptr =
    unsafe { Allocator::allocate_with_metadata(total_size, MIN_ALIGN, true) };
  ptr as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
  let user_ptr = ptr as *mut u8;

  if user_ptr.is_null() {
    return;
  }

  let _ = unsafe { Allocator::deallocate_with_metadata(user_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aligned_alloc(
  alignment: usize,
  size: usize,
) -> *mut c_void {
  if !alignment.is_power_of_two() || alignment == 0 {
    return ptr::null_mut();
  }

  if size % alignment != 0 {
    return ptr::null_mut();
  }

  let ptr =
    unsafe { Allocator::allocate_with_metadata(size, alignment, false) };
  ptr as *mut c_void
}

#[cfg(unix)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_memalign(
  memptr: *mut *mut c_void,
  alignment: usize,
  size: usize,
) -> c_int {
  if memptr.is_null() {
    return libc::EINVAL;
  }

  if alignment == 0 {
    unsafe {
      *memptr = ptr::null_mut();
    }
    return libc::EINVAL;
  }

  if !alignment.is_power_of_two() {
    unsafe {
      *memptr = ptr::null_mut();
    }
    return libc::EINVAL;
  }

  if alignment % mem::size_of::<*mut c_void>() != 0 {
    unsafe {
      *memptr = ptr::null_mut();
    }
    return libc::EINVAL;
  }

  let ptr =
    unsafe { Allocator::allocate_with_metadata(size, alignment, false) };
  if ptr.is_null() {
    unsafe {
      *memptr = ptr::null_mut();
    }
    return libc::ENOMEM;
  }

  unsafe {
    *memptr = ptr as *mut c_void;
  }
  0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
  let user_ptr = ptr as *mut u8;

  if user_ptr.is_null() {
    return unsafe { malloc(size) };
  }

  if size == 0 {
    unsafe { free(ptr) };
    return ZERO_SIZE_PTR as *mut c_void;
  }

  if user_ptr == ZERO_SIZE_PTR {
    return unsafe { malloc(size) };
  }

  let metadata =
    match unsafe { Allocator::validate_and_extract_metadata(user_ptr) } {
      Some(meta) => meta,
      None => return ptr::null_mut(),
    };

  let old_size = Allocator::calculate_user_size(metadata);
  let old_align = metadata.ualign as usize;
  let copy_size = old_size.min(size);

  let new_ptr =
    unsafe { Allocator::allocate_with_metadata(size, old_align, false) };
  if new_ptr.is_null() {
    return ptr::null_mut();
  }

  unsafe { ptr::copy_nonoverlapping(user_ptr, new_ptr, copy_size) };

  let _ = unsafe { Allocator::deallocate_with_metadata(user_ptr) };

  new_ptr as *mut c_void
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc_usable_size(ptr: *mut c_void) -> usize {
  let user_ptr = ptr as *mut u8;

  if user_ptr.is_null() || user_ptr == ZERO_SIZE_PTR {
    return 0;
  }

  let metadata =
    match unsafe { Allocator::validate_and_extract_metadata(user_ptr) } {
      Some(meta) => meta,
      None => return 0,
    };

  Allocator::calculate_user_size(metadata)
}
