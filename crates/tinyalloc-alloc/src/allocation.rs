use std::{
  alloc::Layout,
  mem,
  ptr::NonNull,
};

use getset::CloneGetters;
use tinyalloc_list::{
  HasLink,
  Link,
};

use crate::{
  config::MAX_ALIGN,
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
  layout: Layout,
  #[getset(get_clone = "pub")]
  ptr: NonNull<u8>,
  #[getset(get_clone = "pub")]
  user: NonNull<u8>,
  link: Link<Allocation<'mapper>>,
}

impl<'mapper> Allocation<'mapper> {
  pub fn new(
    owned: AllocationOwner<'mapper>,
    layout: Layout,
    ptr: NonNull<u8>,
    user: NonNull<u8>,
  ) -> Self {
    Self {
      owned,
      layout,
      ptr,
      user,
      link: Link::new(),
    }
  }

  pub fn from(user_ptr: NonNull<u8>) -> Option<NonNull<Self>> {
    let user_addr = user_ptr.as_ptr() as usize;
    if user_addr < mem::size_of::<Self>() + MAX_ALIGN {
      return None;
    }

    let max_header_end = user_addr - MAX_ALIGN + 1;
    let header_start = max_header_end - mem::size_of::<Self>();
    let header_ptr = header_start as *mut Self;
    if header_ptr.is_null() {
      return None;
    }
    unsafe { header_ptr.as_mut() }.map(|h| unsafe { NonNull::new_unchecked(h) })
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
