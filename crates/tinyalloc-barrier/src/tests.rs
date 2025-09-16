use std::{
  sync::Arc,
  thread,
};

use super::Barrier;

#[test]
fn test_same_thread_get() {
  let barrier = Barrier::new(42);
  assert_eq!(*barrier.get().unwrap(), 42);
}

#[test]
fn test_same_thread_get_mut() {
  let barrier = Barrier::new(42);
  *barrier.get_mut().unwrap() = 100;
  assert_eq!(*barrier.get().unwrap(), 100);
}

#[test]
fn test_cross_thread_get_returns_none() {
  let barrier = Arc::new(Barrier::new(42));
  let barrier_clone = barrier.clone();

  let handle = thread::spawn(move || barrier_clone.get().is_some());

  let result = handle.join().unwrap();
  assert!(!result);
}

#[test]
fn test_cross_thread_get_mut_returns_none() {
  let barrier = Arc::new(Barrier::new(42));
  let barrier_clone = barrier.clone();

  let handle = thread::spawn(move || barrier_clone.get_mut().is_some());

  let result = handle.join().unwrap();
  assert!(!result);
}
