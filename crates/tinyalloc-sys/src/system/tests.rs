use crate::{
  size::page_size,
  vm::Mapper,
};

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
fn test_map_single_page() {
  let size = page_size();
  let result = MAPPER.map(size, true);
  assert!(result.is_ok());

  let ptr = result.unwrap();
  assert_eq!(ptr.len(), size);

  MAPPER.unmap(ptr);
}

#[test]
fn test_map_multiple_pages() {
  let size = page_size() * 4;
  let result = MAPPER.map(size, true);
  assert!(result.is_ok());

  let ptr = result.unwrap();
  assert_eq!(ptr.len(), size);

  MAPPER.unmap(ptr);
}

#[test]
fn test_map_write_read() {
  let size = page_size();
  let ptr = MAPPER.map(size, true).unwrap();

  unsafe {
    let slice =
      std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, ptr.len());
    slice[0] = 42;
    slice[size - 1] = 24;

    assert_eq!(slice[0], 42);
    assert_eq!(slice[size - 1], 24);
  }

  MAPPER.unmap(ptr);
}

#[test]
fn test_commit_after_decommit() {
  let size = page_size();
  let ptr = MAPPER.map(size, true).unwrap();

  MAPPER.decommit(ptr).unwrap();

  let result = MAPPER.commit(ptr);
  assert!(result.is_ok());

  unsafe {
    let slice =
      std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, ptr.len());
    slice[0] = 123;
    assert_eq!(slice[0], 123);
  }

  MAPPER.unmap(ptr);
}

#[test]
fn test_decommit() {
  let size = page_size();
  let ptr = MAPPER.map(size, true).unwrap();

  let result = MAPPER.decommit(ptr);
  assert!(result.is_ok());

  MAPPER.unmap(ptr);
}

#[test]
fn test_protect() {
  let size = page_size();
  let ptr = MAPPER.map(size, true).unwrap();

  let result = MAPPER.protect(ptr);
  assert!(result.is_ok());

  MAPPER.unmap(ptr);
}
