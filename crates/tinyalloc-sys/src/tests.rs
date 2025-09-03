use tinyalloc_core::{
  page::Page,
  size::page_size,
  vm::Mapper,
};

#[cfg(unix)]
use crate::posix::PosixMapper;
#[cfg(windows)]
use crate::windows::WindowsMapper;

#[cfg(unix)]
static BACKING_MAPPER: PosixMapper = PosixMapper;

#[cfg(windows)]
static BACKING_MAPPER: WindowsMapper = WindowsMapper;

static MAPPER: &dyn Mapper = &BACKING_MAPPER;

#[test]
fn test_map_single_page() {
  let size = page_size();
  let result = MAPPER.map(size);
  assert!(result.is_ok());

  let ptr = result.unwrap();
  assert_eq!(ptr.len(), size);

  MAPPER.unmap(ptr);
}

#[test]
fn test_map_multiple_pages() {
  let size = page_size() * 4;
  let result = MAPPER.map(size);
  assert!(result.is_ok());

  let ptr = result.unwrap();
  assert_eq!(ptr.len(), size);

  MAPPER.unmap(ptr);
}

#[test]
fn test_map_write_read() {
  let size = page_size();
  let ptr = MAPPER.map(size).unwrap();

  unsafe {
    let slice = std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, ptr.len());
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
  let ptr = MAPPER.map(size).unwrap();

  MAPPER.decommit(ptr).unwrap();

  let result = MAPPER.commit(ptr);
  assert!(result.is_ok());

  unsafe {
    let slice = std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, ptr.len());
    slice[0] = 123;
    assert_eq!(slice[0], 123);
  }

  MAPPER.unmap(ptr);
}

#[test]
fn test_decommit() {
  let size = page_size();
  let ptr = MAPPER.map(size).unwrap();

  let result = MAPPER.decommit(ptr);
  assert!(result.is_ok());

  MAPPER.unmap(ptr);
}

#[test]
fn test_protect() {
  let size = page_size();
  let ptr = MAPPER.map(size).unwrap();

  let result = MAPPER.protect(ptr);
  assert!(result.is_ok());

  MAPPER.unmap(ptr);
}

#[test]
fn test_page_raii_basic() {
  let page = Page::new(MAPPER, page_size());
  assert!(page.is_ok());
  assert!(page.unwrap().is_mapped());
}

#[test]
fn test_page_raii_operations() {
  let mut page = Page::new(MAPPER, page_size()).unwrap();

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
  let mut page = Page::new(MAPPER, page_size()).unwrap();

  page.decommit().unwrap();
  assert!(!page.is_committed());

  page.commit().unwrap();
  assert!(page.is_committed());

  if page.is_committed() && !page.is_protected() {
    page.as_mut()[0] = 42;
    assert_eq!(page.as_ref()[0], 42);
  }
}
