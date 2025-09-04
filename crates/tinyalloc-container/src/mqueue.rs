use std::ptr::NonNull;

use getset::{
  Getters,
  MutGetters,
};
use tinyalloc_sys::{
  page::Page,
  size::page_size,
  vm::{
    MapError,
    Mapper,
  },
};

pub const QUEUE_NODE_ALIGNMENT: usize = 8;
pub const QUEUE_PAGE_MULTIPLIER: usize = 8;

#[derive(Getters, MutGetters)]
pub struct Node<T> {
  #[getset(get = "pub", get_mut = "pub")]
  value: T,
  #[getset(get = "pub", get_mut = "pub")]
  next: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
  pub fn new(value: T) -> Self {
    Self { value, next: None }
  }
}

pub struct MappedQueue<'mapper, T, const N: usize = 8> {
  head: Option<NonNull<Node<T>>>,
  tail: Option<NonNull<Node<T>>>,
  len: usize,
  data: Option<Page<'mapper>>,
  allocated_nodes: usize,
  system: &'mapper dyn Mapper,
}

impl<'mapper, T, const N: usize> MappedQueue<'mapper, T, N> {
  pub fn new(system: &'mapper dyn Mapper) -> Self {
    Self {
      head: None,
      tail: None,
      len: 0,
      data: None,
      allocated_nodes: 0,
      system,
    }
  }

  fn nodes_per_page(&self) -> usize {
    (page_size() * N) / std::mem::size_of::<Node<T>>()
  }

  fn ensure_capacity(&mut self) -> Result<(), MapError> {
    if self.data.is_none() || self.allocated_nodes >= self.nodes_per_page() {
      let queue_page_size = page_size() * N;
      let new_page = Page::new(self.system, queue_page_size)?;
      self.data = Some(new_page);
      self.allocated_nodes = 0;
    }
    Ok(())
  }

  fn allocate_node(&mut self, value: T) -> Result<NonNull<Node<T>>, MapError> {
    self.ensure_capacity()?;

    let page = self.data.as_mut().unwrap();
    let page_slice = page.as_mut();

    unsafe {
      let base_ptr = page_slice.as_mut_ptr() as *mut Node<T>;
      let node_ptr = base_ptr.add(self.allocated_nodes);

      std::ptr::write(node_ptr, Node::new(value));
      self.allocated_nodes += 1;

      Ok(NonNull::new_unchecked(node_ptr))
    }
  }

  pub fn push(&mut self, value: T) -> Result<(), MapError> {
    let new_node = self.allocate_node(value)?;

    unsafe {
      if let Some(tail) = self.tail {
        (*tail.as_ptr()).next = Some(new_node);
      } else {
        self.head = Some(new_node);
      }

      self.tail = Some(new_node);
      self.len += 1;
    }

    Ok(())
  }

  pub fn pop(&mut self) -> Option<T> {
    let head = self.head?;

    unsafe {
      let head_ref = head.as_ref();
      let value = std::ptr::read(&head_ref.value);

      self.head = head_ref.next;

      if self.head.is_none() {
        self.tail = None;
      }

      self.len = self.len.saturating_sub(1);

      Some(value)
    }
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }
}

impl<'mapper, T, const N: usize> Drop for MappedQueue<'mapper, T, N> {
  fn drop(&mut self) {
    while !self.is_empty() {
      let _ = self.pop();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tinyalloc_sys::vm::Mapper;

  #[cfg(unix)]
  use tinyalloc_sys::system::posix::PosixMapper;
  #[cfg(windows)]
  use tinyalloc_sys::system::windows::WindowsMapper;

  #[cfg(unix)]
  static BACKING_MAPPER: PosixMapper = PosixMapper;
  #[cfg(windows)]
  static BACKING_MAPPER: WindowsMapper = WindowsMapper;

  static MAPPER: &dyn Mapper = &BACKING_MAPPER;

  #[test]
  fn test_new() {
    let queue: MappedQueue<i32> = MappedQueue::new(MAPPER);
    assert_eq!(queue.len(), 0);
    assert!(queue.is_empty());
  }

  #[test]
  fn test_push_pop() {
    let mut queue: MappedQueue<i32> = MappedQueue::new(MAPPER);

    queue.push(1).unwrap();
    queue.push(2).unwrap();

    assert_eq!(queue.len(), 2);
    assert_eq!(queue.pop(), Some(1));
    assert_eq!(queue.pop(), Some(2));
    assert_eq!(queue.pop(), None);
  }

  #[test]
  fn test_fifo_order() {
    let mut queue: MappedQueue<i32> = MappedQueue::new(MAPPER);

    for i in 0..10 {
      queue.push(i).unwrap();
    }

    for i in 0..10 {
      assert_eq!(queue.pop(), Some(i));
    }
  }
}
