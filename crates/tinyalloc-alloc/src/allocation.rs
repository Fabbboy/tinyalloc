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
  config::{
    align_up,
    MAX_ALIGN,
  },
  heap::Heap,
};

const ALLOCATION_CANARY: u64 = 0xDEADBEEFCAFEBABE;

#[derive(Clone)]
pub enum AllocationOwner<'mapper> {
  Heap(*mut Heap<'mapper>),
  Mapper(NonNull<[u8]>),
}

#[derive(CloneGetters)]
pub struct Allocation<'mapper> {
  #[getset(get_clone = "pub")]
  owned: AllocationOwner<'mapper>,
  #[getset(get_clone = "pub")]
  full: Layout,
  #[getset(get_clone = "pub")]
  alloc_ptr: *mut u8,
  #[getset(get_clone = "pub")]
  user_ptr: *mut u8,
  canary: u64,
  link: Link<Allocation<'mapper>>,
}

impl<'mapper> Allocation<'mapper> {
  pub fn new(
    owned: AllocationOwner<'mapper>,
    full: Layout,
    alloc_ptr: *mut u8,
    user_ptr: *mut u8,
  ) -> Self {
    Self {
      owned,
      full,
      alloc_ptr,
      user_ptr,
      canary: ALLOCATION_CANARY,
      link: Link::new(),
    }
  }

  pub fn from(ptr: *mut u8) -> Option<*mut Self> {
    if ptr.is_null() {
      return None;
    }

    let user_addr = ptr as usize;
    if user_addr < mem::size_of::<Self>() + MAX_ALIGN {
      return None;
    }

    let header_size = mem::size_of::<Self>();
    let max_header_end = match user_addr.checked_sub(MAX_ALIGN - 1) {
      Some(end) => end,
      None => return None,
    };

    let header_start = match max_header_end.checked_sub(header_size) {
      Some(start) => start,
      None => return None,
    };

    if header_start % mem::align_of::<Self>() != 0 {
      return None;
    }

    let header_ptr = header_start as *mut Self;

    if header_ptr.is_null() {
      return None;
    }

    let allocation = unsafe { &*header_ptr };
    if allocation.canary != ALLOCATION_CANARY {
      return None;
    }

    Some(header_ptr)
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

  pub unsafe fn heap_ptr(&self) -> Option<&Heap<'mapper>> {
    match self.owned {
      AllocationOwner::Heap(heap_ptr) => Some(unsafe { &*heap_ptr }),
      AllocationOwner::Mapper(_) => None,
    }
  }

  pub unsafe fn map_range(&self) -> Option<NonNull<[u8]>> {
    match self.owned {
      AllocationOwner::Mapper(ref slice_ptr) => Some(*slice_ptr),
      AllocationOwner::Heap(_) => None,
    }
  }

  pub fn thread(&self) -> Option<ThreadId> {
    match unsafe { self.heap_ptr() } {
      Some(heap) => Some(heap.thread()),
      None => None,
    }
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
