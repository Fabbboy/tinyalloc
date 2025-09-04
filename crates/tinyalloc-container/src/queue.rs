use std::ptr::NonNull;

use getset::{
  Getters,
  MutGetters,
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

pub struct Queue<T> {
  head: Option<NonNull<Node<T>>>,
  current: Option<NonNull<Node<T>>>,
  len: usize,
}

impl<T> Queue<T> {
  pub fn new() -> Self {
    Self {
      head: None,
      current: None,
      len: 0,
    }
  }
}
