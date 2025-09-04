use std::ptr::NonNull;

use getset::{
  Getters,
  MutGetters,
};

// Philosophy: A non-owning singly linked list

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

  pub fn push(&mut self, node: NonNull<Node<T>>) {
    unsafe {
      if let Some(mut current) = self.current {
        current.as_mut().next = Some(node);
        self.current = Some(node);
      } else {
        self.head = Some(node);
        self.current = Some(node);
      }
      self.len += 1;
    }
  }

  pub fn pop(&mut self) -> Option<NonNull<Node<T>>> {
    if let Some(head) = self.head {
      unsafe {
        self.head = head.as_ref().next;
        if self.head.is_none() {
          self.current = None;
        }
      }
      self.len -= 1;
      Some(head)
    } else {
      None
    }
  }
}
