use super::*;

#[test]
fn test_array_new() {
  let arr: Array<i32, 4> = Array::new();

  assert_eq!(arr.len(), 0);
  assert_eq!(arr.capacity(), 4);
  assert!(arr.is_empty());
  assert!(!arr.is_full());
}

#[test]
fn test_array_push_pop() {
  let mut arr: Array<i32, 3> = Array::new();

  assert!(arr.push(1).is_ok());
  assert!(arr.push(2).is_ok());
  assert_eq!(arr.len(), 2);
  assert!(!arr.is_empty());

  assert_eq!(arr.pop(), Some(2));
  assert_eq!(arr.pop(), Some(1));
  assert_eq!(arr.len(), 0);
  assert!(arr.is_empty());
  assert_eq!(arr.pop(), None);
}

#[test]
fn test_array_capacity_exceeded() {
  let mut arr: Array<i32, 2> = Array::new();

  assert!(arr.push(1).is_ok());
  assert!(arr.push(2).is_ok());
  assert!(arr.is_full());

  let result = arr.push(3);
  assert!(result.is_err());
  match result {
    Err(ArrayError::InsufficientCapacity { have, need }) => {
      assert_eq!(have, 2);
      assert_eq!(need, 3);
    }
    _ => panic!("Expected InsufficientCapacity error"),
  }
}

#[test]
fn test_array_get() {
  let mut arr: Array<i32, 4> = Array::new();

  arr.push(10).unwrap();
  arr.push(20).unwrap();

  assert_eq!(*arr.get(0).unwrap(), 10);
  assert_eq!(*arr.get(1).unwrap(), 20);

  let result = arr.get(2);
  assert!(result.is_err());
  match result {
    Err(ArrayError::OutOfBounds { index, size }) => {
      assert_eq!(index, 2);
      assert_eq!(size, 2);
    }
    _ => panic!("Expected OutOfBounds error"),
  }
}

#[test]
fn test_array_get_mut() {
  let mut arr: Array<i32, 4> = Array::new();

  arr.push(10).unwrap();
  arr.push(20).unwrap();

  *arr.get_mut(0).unwrap() = 100;
  assert_eq!(*arr.get(0).unwrap(), 100);

  let result = arr.get_mut(2);
  assert!(result.is_err());
  match result {
    Err(ArrayError::OutOfBounds { index, size }) => {
      assert_eq!(index, 2);
      assert_eq!(size, 2);
    }
    _ => panic!("Expected OutOfBounds error"),
  }
}

#[test]
fn test_array_clear() {
  let mut arr: Array<i32, 4> = Array::new();

  arr.push(1).unwrap();
  arr.push(2).unwrap();
  arr.push(3).unwrap();
  assert_eq!(arr.len(), 3);

  arr.clear();
  assert_eq!(arr.len(), 0);
  assert!(arr.is_empty());
}

#[test]
fn test_array_deref() {
  let mut arr: Array<i32, 4> = Array::new();

  arr.push(1).unwrap();
  arr.push(2).unwrap();
  arr.push(3).unwrap();

  let slice_ref: &[i32] = &arr;
  assert_eq!(slice_ref.len(), 3);
  assert_eq!(slice_ref[0], 1);
  assert_eq!(slice_ref[1], 2);
  assert_eq!(slice_ref[2], 3);
}

#[test]
fn test_array_as_slice() {
  let mut arr: Array<i32, 4> = Array::new();

  arr.push(10).unwrap();
  arr.push(20).unwrap();

  let s = arr.as_slice();
  assert_eq!(s.len(), 2);
  assert_eq!(s[0], 10);
  assert_eq!(s[1], 20);
}

#[test]
fn test_array_as_mut_slice() {
  let mut arr: Array<i32, 4> = Array::new();

  arr.push(10).unwrap();
  arr.push(20).unwrap();

  let s = arr.as_mut_slice();
  s[0] = 100;
  assert_eq!(*arr.get(0).unwrap(), 100);
}

#[test]
fn test_array_unsafe_get() {
  let mut arr: Array<i32, 4> = Array::new();

  arr.push(42).unwrap();

  unsafe {
    assert_eq!(*arr.get_unchecked(0), 42);
    *arr.get_unchecked_mut(0) = 84;
    assert_eq!(*arr.get_unchecked(0), 84);
  }
}

#[test]
fn test_array_default() {
  let arr: Array<i32, 4> = Array::default();
  assert_eq!(arr.len(), 0);
  assert_eq!(arr.capacity(), 4);
}

#[test]
fn test_array_zero_capacity() {
  let mut arr: Array<i32, 0> = Array::new();

  assert_eq!(arr.capacity(), 0);
  assert!(arr.is_empty());
  assert!(arr.is_full());

  let result = arr.push(1);
  assert!(result.is_err());
  assert_eq!(arr.pop(), None);
}
