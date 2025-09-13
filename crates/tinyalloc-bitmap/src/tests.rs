use super::*;

#[test]
fn test_basic_bit_operations() {
    let mut bitmap: Bitmap<u32, 2> = Bitmap::new();

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
    let mut bitmap: Bitmap<u64, 2> = Bitmap::new();

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
    let mut bitmap: Bitmap<u32, 3> = Bitmap::new();

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
    let mut bitmap: Bitmap<u32, 2> = Bitmap::new();

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
    let mut bitmap: Bitmap<u32, 1> = Bitmap::new();

    assert!(bitmap.set(31).is_ok());
    assert!(bitmap.set(32).is_err());
    assert!(bitmap.get(32).is_err());
    assert!(bitmap.clear(32).is_err());
    assert!(bitmap.flip(32).is_err());

    let result = Bitmap::<u32, 1>::check(64);
    assert!(result.is_err());
}

#[test]
fn test_partial_word_handling() {
    let mut bitmap: Bitmap<u32, 1> = Bitmap::new();

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
    let mut bitmap8: Bitmap<u8, 2> = Bitmap::new();
    bitmap8.set(7).unwrap();
    bitmap8.set(8).unwrap();
    assert!(bitmap8.get(7).unwrap());
    assert!(bitmap8.get(8).unwrap());

    let mut bitmap16: Bitmap<u16, 1> = Bitmap::new();
    bitmap16.set(9).unwrap();
    assert!(bitmap16.get(9).unwrap());
    assert_eq!(bitmap16.find_first_set(), Some(9));
}