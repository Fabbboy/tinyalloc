use super::*;

#[test]
fn test_basic_bit_operations() {
  let mut storage: [u32; 2] = [0; 2];
  let mut bitmap =
    Bitmap::zero(&mut storage, storage.len() * u32::BITS as usize).unwrap();

  assert!(!bitmap.get(0).unwrap());
  assert!(!bitmap.get(31).unwrap());

  bitmap.set(0).unwrap();
  assert!(bitmap.get(0).unwrap());
  assert!(!bitmap.get(1).unwrap());

  bitmap.set(31).unwrap();
  assert!(bitmap.get(31).unwrap());

  bitmap.clear(0).unwrap();
  assert!(!bitmap.get(0).unwrap());
  assert!(bitmap.get(31).unwrap());

  bitmap.flip(0).unwrap();
  assert!(bitmap.get(0).unwrap());

  bitmap.flip(0).unwrap();
  assert!(!bitmap.get(0).unwrap());
}

#[test]
fn test_multi_word_operations() {
  let mut storage: [u64; 2] = [0; 2];
  let mut bitmap =
    Bitmap::zero(&mut storage, storage.len() * u64::BITS as usize).unwrap();

  bitmap.set(0).unwrap();
  bitmap.set(63).unwrap();
  bitmap.set(64).unwrap();
  bitmap.set(99).unwrap();

  assert!(bitmap.get(0).unwrap());
  assert!(bitmap.get(63).unwrap());
  assert!(bitmap.get(64).unwrap());
  assert!(bitmap.get(99).unwrap());
  assert!(!bitmap.get(32).unwrap());
  assert!(!bitmap.get(96).unwrap());
}

#[test]
fn test_bulk_operations() {
  let mut storage: [u32; 3] = [0; 3];
  let mut bitmap =
    Bitmap::zero(&mut storage, storage.len() * u32::BITS as usize).unwrap();

  bitmap.set(5).unwrap();
  bitmap.set(35).unwrap();
  bitmap.set(65).unwrap();

  assert!(bitmap.get(5).unwrap());
  assert!(bitmap.get(35).unwrap());
  assert!(bitmap.get(65).unwrap());

  bitmap.clear_all();
  assert!(!bitmap.get(5).unwrap());
  assert!(!bitmap.get(35).unwrap());
  assert!(!bitmap.get(65).unwrap());

  bitmap.set_all();
  assert!(bitmap.get(0).unwrap());
  assert!(bitmap.get(31).unwrap());
  assert!(bitmap.get(32).unwrap());
  assert!(bitmap.get(63).unwrap());
  assert!(bitmap.get(64).unwrap());
  assert!(bitmap.get(95).unwrap()); // Changed from 79 to 95 since we have 3 * 32 = 96 bits
}

#[test]
fn test_search_operations() {
  let mut storage: [u32; 2] = [0; 2];
  let mut bitmap =
    Bitmap::zero(&mut storage, storage.len() * u32::BITS as usize).unwrap();

  assert_eq!(bitmap.find_first_set(), None);
  assert_eq!(bitmap.find_first_clear(), Some(0));

  bitmap.set(5).unwrap();
  bitmap.set(35).unwrap();

  assert_eq!(bitmap.find_first_set(), Some(5));
  assert_eq!(bitmap.find_first_clear(), Some(0));

  bitmap.set(0).unwrap();
  assert_eq!(bitmap.find_first_clear(), Some(1));

  bitmap.set_all();
  assert_eq!(bitmap.find_first_clear(), None);
  assert_eq!(bitmap.find_first_set(), Some(0));
}

#[test]
fn test_error_handling() {
  let mut storage: [u32; 1] = [0; 1];
  let err = Bitmap::zero(&mut storage, storage.len() * u32::BITS as usize + 1);
  assert!(matches!(
    err,
    Err(BitmapError::InsufficientSize { have, need }) if have
      < need
  ));

  let mut bitmap =
    Bitmap::zero(&mut storage, storage.len() * u32::BITS as usize).unwrap();

  assert!(bitmap.set(31).is_ok());
  assert!(bitmap.set(32).is_err());
  assert!(bitmap.get(32).is_err());
  assert!(bitmap.clear(32).is_err());
  assert!(bitmap.flip(32).is_err());

  let result = bitmap.check(64);
  assert!(result.is_err());
}

#[test]
fn test_partial_word_handling() {
  let mut storage: [u32; 1] = [0; 1];
  let mut bitmap =
    Bitmap::zero(&mut storage, storage.len() * u32::BITS as usize).unwrap();

  bitmap.set_all();
  for i in 0..32 {
    assert!(bitmap.get(i).unwrap());
  }

  bitmap.clear_all();
  for i in 0..32 {
    assert!(!bitmap.get(i).unwrap());
  }

  bitmap.set(31).unwrap();
  assert_eq!(bitmap.find_first_set(), Some(31));
}

#[test]
fn test_different_word_types() {
  let mut storage8: [u8; 2] = [0; 2];
  let mut bitmap8 =
    Bitmap::zero(&mut storage8, storage8.len() * u8::BITS as usize).unwrap();
  bitmap8.set(7).unwrap();
  bitmap8.set(8).unwrap();
  assert!(bitmap8.get(7).unwrap());
  assert!(bitmap8.get(8).unwrap());

  let mut storage16: [u16; 1] = [0; 1];
  let mut bitmap16 =
    Bitmap::zero(&mut storage16, storage16.len() * u16::BITS as usize).unwrap();
  bitmap16.set(9).unwrap();
  assert!(bitmap16.get(9).unwrap());
  assert_eq!(bitmap16.find_first_set(), Some(9));
}

#[test]
fn test_zero_and_one_constructors() {
  let mut storage: [u32; 2] = [0; 2];

  // Test zero constructor
  let bitmap_zero =
    Bitmap::zero(&mut storage, storage.len() * u32::BITS as usize).unwrap();
  assert!(bitmap_zero.is_clear());
  assert_eq!(bitmap_zero.find_first_set(), None);
  assert_eq!(bitmap_zero.find_first_clear(), Some(0));

  // Test one constructor
  let mut storage2: [u32; 2] = [0; 2];
  let bitmap_one =
    Bitmap::one(&mut storage2, storage2.len() * u32::BITS as usize).unwrap();
  assert!(!bitmap_one.is_clear());
  assert_eq!(bitmap_one.find_first_set(), Some(0));
  assert_eq!(bitmap_one.find_first_clear(), None);
}
