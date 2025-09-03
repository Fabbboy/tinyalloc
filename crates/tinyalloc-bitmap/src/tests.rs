use crate::{
  bitmap::Bitmap,
  error::BitmapError,
};

#[test]
fn test_basic_operations() {
  let mut store = [0u32; 2];
  let mut bitmap = Bitmap::within(&mut store, 50).unwrap();

  // Set and get
  bitmap.set(0).unwrap();
  bitmap.set(31).unwrap();
  bitmap.set(32).unwrap();
  assert!(bitmap.get(0).unwrap());
  assert!(bitmap.get(31).unwrap());
  assert!(bitmap.get(32).unwrap());
  assert!(!bitmap.get(1).unwrap());

  // Clear
  bitmap.clear(0).unwrap();
  assert!(!bitmap.get(0).unwrap());

  // Flip
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

  // Insufficient size
  assert!(matches!(
    Bitmap::within(&mut store, 50),
    Err(BitmapError::InsufficientSize { .. })
  ));

  // Out of bounds
  let mut bitmap = Bitmap::within(&mut store, 20).unwrap();
  assert!(matches!(
    bitmap.get(32),
    Err(BitmapError::OutOfBounds { .. })
  ));
}
