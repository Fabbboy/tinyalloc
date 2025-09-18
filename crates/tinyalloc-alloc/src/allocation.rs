use std::{
  alloc::Layout,
  mem,
  ptr::NonNull,
  thread::ThreadId,
};

use getset::CloneGetters;
use tinyalloc_list::{
  HasLink,
  Link,
};

use crate::{
  config::{align_up, MAX_ALIGN},
  heap::Heap,
};

#[derive(Clone)]
pub enum AllocationOwner<'mapper> {
  Heap(NonNull<Heap<'mapper>>),
  Mapper,
}

#[derive(CloneGetters)]
pub struct Allocation<'mapper> {
  #[getset(get_clone = "pub")]
  owned: AllocationOwner<'mapper>,
  #[getset(get_clone = "pub")]
  full: Layout,
  #[getset(get_clone = "pub")]
  alloc_ptr: NonNull<u8>,
  #[getset(get_clone = "pub")]
  user_ptr: NonNull<u8>,
  link: Link<Allocation<'mapper>>,
}

impl<'mapper> Allocation<'mapper> {
  pub fn new(
    owned: AllocationOwner<'mapper>,
    full: Layout,
    ptr: NonNull<u8>,
    user: NonNull<u8>,
  ) -> Self {
    Self {
      owned,
      full,
      alloc_ptr: ptr,
      user_ptr: user,
      link: Link::new(),
    }
  }

  pub fn from(ptr: *mut u8) -> Option<NonNull<Self>> {
    if ptr.is_null() {
      return None;
    }

    let user_addr = ptr as usize;
    if user_addr < mem::size_of::<Self>() + MAX_ALIGN {
      return None;
    }

    let max_header_end = user_addr - MAX_ALIGN + 1;
    let header_start = max_header_end - mem::size_of::<Self>();
    let header_ptr = header_start as *mut Self;

    NonNull::new(header_ptr)
  }

  pub fn get_user_ptr(&self) -> *mut u8 {
    self.user_ptr().as_ptr()
  }

  pub fn total_size(user_layout: Layout) -> usize {
    let header_size = mem::size_of::<Self>();
    let user_size = user_layout.size();
    let padding = MAX_ALIGN - 1;
    header_size + padding + user_size
  }

  pub fn calc_user_ptr(header_ptr: *const Self) -> *mut u8 {
    let header_addr = header_ptr as usize;
    let user_addr = align_up(header_addr + mem::size_of::<Self>(), MAX_ALIGN);
    user_addr as *mut u8
  }

  pub fn heap_ptr(&self) -> Option<NonNull<Heap<'mapper>>> {
    match &self.owned {
      AllocationOwner::Heap(heap_ptr) => Some(*heap_ptr),
      AllocationOwner::Mapper => None,
    }
  }

  pub fn thread(&self) -> Option<ThreadId> {
    self.heap_ptr().map(|heap_ptr| unsafe { heap_ptr.as_ref().thread() })
  }
}

impl<'mapper> HasLink<Allocation<'mapper>> for Allocation<'mapper> {
  fn link(&self) -> &Link<Self> {
    &self.link
  }

  fn link_mut(&mut self) -> &mut Link<Self> {
    &mut self.link
  }
}
