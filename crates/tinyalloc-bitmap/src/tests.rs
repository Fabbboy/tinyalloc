use super::*;

#[test]
fn test_basic_bit_operations() {
    let mut storage = [0u32; 2];
    let mut bitmap = Bitmap::within(&mut storage, 32).unwrap();

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
    let mut storage = [0u64; 2];
    let mut bitmap = Bitmap::within(&mut storage, 100).unwrap();

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
    let mut storage = [0u32; 3];
    let mut bitmap = Bitmap::within(&mut storage, 80).unwrap();

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
    assert!(bitmap.get(79).unwrap());
}

#[test]
fn test_search_operations() {
    let mut storage = [0u32; 2];
    let mut bitmap = Bitmap::within(&mut storage, 64).unwrap();

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
    let mut storage = [0u32; 1];
    let mut bitmap = Bitmap::within(&mut storage, 16).unwrap();

    assert!(bitmap.set(15).is_ok());
    assert!(bitmap.set(16).is_err());
    assert!(bitmap.get(16).is_err());
    assert!(bitmap.clear(16).is_err());
    assert!(bitmap.flip(16).is_err());

    let result = Bitmap::within(&mut storage, 64);
    assert!(result.is_err());
}

#[test]
fn test_partial_word_handling() {
    let mut storage = [0u32; 1];
    let mut bitmap = Bitmap::within(&mut storage, 20).unwrap();

    bitmap.set_all();
    for i in 0..20 {
        assert!(bitmap.get(i).unwrap());
    }

    bitmap.clear_all();
    for i in 0..20 {
        assert!(!bitmap.get(i).unwrap());
    }

    bitmap.set(19).unwrap();
    assert_eq!(bitmap.find_first_set(), Some(19));
}

#[test]
fn test_different_word_types() {
    let mut storage8 = [0u8; 2];
    let mut bitmap8 = Bitmap::within(&mut storage8, 12).unwrap();
    bitmap8.set(7).unwrap();
    bitmap8.set(8).unwrap();
    assert!(bitmap8.get(7).unwrap());
    assert!(bitmap8.get(8).unwrap());

    let mut storage16 = [0u16; 1];
    let mut bitmap16 = Bitmap::within(&mut storage16, 10).unwrap();
    bitmap16.set(9).unwrap();
    assert!(bitmap16.get(9).unwrap());
    assert_eq!(bitmap16.find_first_set(), Some(9));
}