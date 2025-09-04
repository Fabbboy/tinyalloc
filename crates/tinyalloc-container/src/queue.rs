use std::ptr::NonNull;

use getset::{
  Getters,
  MutGetters,
};
use tinyalloc_sys::{
  page::Page,
  vm::Mapper,
};

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

pub struct Queue<'mapper, T> {
  head: Option<NonNull<Node<T>>>,
  current: Option<NonNull<Node<T>>>,
  len: usize,
  data: Option<Page<'mapper>>, // everything is allocated inside here and the arena is **REPLACED** when more space is needed because most OS do not support remapping. Additionally, this is an Option **ONLY** allocated when something is actually pushed
  system: &'mapper dyn Mapper,
}

impl<'mapper, T> Queue<'mapper, T> {
  pub fn new(system: &'mapper dyn Mapper) -> Self {
    Self {
      head: None,
      current: None,
      len: 0,
      data: None,
      system,
    }
  }

  pub fn push(&mut self, value: T) {}
  pub fn pop(&mut self) -> Option<T> {
    None
  }
}
