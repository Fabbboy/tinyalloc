use crate::{
  bitmap::Bitmap,
  error::BitmapError,
};

#[test]
fn test_basic_operations() {
  let mut store = [0u32; 2];
  let mut bitmap = Bitmap::within(&mut store, 50).unwrap();

  bitmap.set(0).unwrap();
  bitmap.set(31).unwrap();
  bitmap.set(32).unwrap();
  assert!(bitmap.get(0).unwrap());
  assert!(bitmap.get(31).unwrap());
  assert!(bitmap.get(32).unwrap());
  assert!(!bitmap.get(1).unwrap());

  bitmap.clear(0).unwrap();
  assert!(!bitmap.get(0).unwrap());

  bitmap.flip(1).unwrap();
  assert!(bitmap.get(1).unwrap());
  bitmap.flip(1).unwrap();
  assert!(!bitmap.get(1).unwrap());
}

#[test]
fn test_clear_and_set_all() {
  let mut store = [0u32; 2];
  let mut bitmap = Bitmap::within(&mut store, 50).unwrap();

  bitmap.set_all();
  assert!(bitmap.get(0).unwrap());
  assert!(bitmap.get(31).unwrap());

  bitmap.clear_all();
  assert!(!bitmap.get(0).unwrap());
  assert!(!bitmap.get(31).unwrap());
}

#[test]
fn test_errors() {
  let mut store = [0u32; 1];

  assert!(matches!(
    Bitmap::within(&mut store, 50),
    Err(BitmapError::InsufficientSize { .. })
  ));

  let bitmap = Bitmap::within(&mut store, 20).unwrap();
  assert!(matches!(
    bitmap.get(32),
    Err(BitmapError::OutOfBounds { .. })
  ));
}

#[test]
fn test_first_set() {
  let mut store = [0u32; 2];
  let mut bitmap = Bitmap::within(&mut store, 50).unwrap();

  assert_eq!(bitmap.first_set(), None);

  bitmap.set(0).unwrap();
  assert_eq!(bitmap.first_set(), Some(0));

  bitmap.clear_all();
  bitmap.set(15).unwrap();
  assert_eq!(bitmap.first_set(), Some(15));

  bitmap.clear_all();
  bitmap.set(35).unwrap();
  assert_eq!(bitmap.first_set(), Some(35));

  bitmap.set(10).unwrap();
  bitmap.set(45).unwrap();
  assert_eq!(bitmap.first_set(), Some(10));

  bitmap.clear_all();
  bitmap.set(31).unwrap();
  assert_eq!(bitmap.first_set(), Some(31));

  bitmap.clear_all();
  bitmap.set(32).unwrap();
  assert_eq!(bitmap.first_set(), Some(32));
}

#[test]
fn test_first_unset() {
  let mut store = [0u32; 2];
  let available_size = Bitmap::available(&store);
  let mut bitmap = Bitmap::within(&mut store, available_size).unwrap();

  assert_eq!(bitmap.first_unset(), Some(0));

  bitmap.set(0).unwrap();
  assert_eq!(bitmap.first_unset(), Some(1));

  bitmap.set_all();
  bitmap.clear(15).unwrap();
  assert_eq!(bitmap.first_unset(), Some(15));

  bitmap.set_all();
  bitmap.clear(35).unwrap();
  assert_eq!(bitmap.first_unset(), Some(35));

  bitmap.set_all();
  bitmap.clear(10).unwrap();
  bitmap.clear(45).unwrap();
  assert_eq!(bitmap.first_unset(), Some(10));

  bitmap.set_all();
  bitmap.clear(31).unwrap();
  assert_eq!(bitmap.first_unset(), Some(31));

  bitmap.set_all();
  bitmap.clear(32).unwrap();
  assert_eq!(bitmap.first_unset(), Some(32));

  bitmap.set_all();
  assert_eq!(bitmap.first_unset(), None);
}
