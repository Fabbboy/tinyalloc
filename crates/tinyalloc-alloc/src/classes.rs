use std::mem;

use crate::{
  CUT_OFF,
  QUANTUM,
  SPAN_SIZE,
};
use getset::Getters;

#[derive(Getters)]
pub struct SizeClass {
  #[getset(get = "pub")]
  class_size: usize,
  #[getset(get = "pub")]
  element_align: usize,
  #[getset(get = "pub")]
  elems: usize,
}

#[inline]
pub const fn next_pow2(n: usize) -> usize {
  let mut n = n - 1;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;
  if mem::size_of::<usize>() == 8 {
    n |= n >> 32;
  }
  n + 1
}

macro_rules! size_classes {
    ($($size:expr),* $(,)?) => {
        &[
            $(
                {
                    const SIZE: usize = $size;
                    const ELEMS: usize = SPAN_SIZE / SIZE;

                    const POW2: usize = next_pow2(SIZE);
                    const ALIGN: usize =
                        if QUANTUM > POW2 {
                            QUANTUM
                        } else if POW2 > SIZE {
                            POW2
                        } else {
                            SIZE
                        };

                    SizeClass {
                        class_size: SIZE,
                        element_align: ALIGN,
                        elems: ELEMS,
                    }
                }
            ),*
        ]
    };
}

#[rustfmt::skip]
pub const SIZE_CLASSES: &[SizeClass] = size_classes![
  // smallest
  QUANTUM, 32, 48, 64, 80, 96, 112, 128, 
  // step 32
  160, 192, 224, 256,
  // step 64
  320, 384, 448, 512,
  // step 128
  640, 768, 896, 1024,
  // step 256
  1280, 1536, 1792, 2048,
  // step 512
  2560, 3072, 3584, 4096,
  // step 1024
  5120, 6144, 7168, 8192,
  // step 2048
  10240, 12288, 14336, 16384,
  // step 4096
  20480, 24576, 28672, 32768,
  // step 8192
  40960, 49152, 57344, // cant have the last one as it is EXACT CUT_OFF which prevents from having a header in the span
];

#[inline]
pub fn find_size_class(size: usize) -> Option<&'static SizeClass> {
  if size > CUT_OFF {
    return None;
  }

  for class in SIZE_CLASSES {
    if class.class_size >= size {
      return Some(class);
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_next_pow2() {
    assert_eq!(next_pow2(1), 1);
    assert_eq!(next_pow2(2), 2);
    assert_eq!(next_pow2(3), 4);
    assert_eq!(next_pow2(4), 4);
    assert_eq!(next_pow2(5), 8);
    assert_eq!(next_pow2(8), 8);
    assert_eq!(next_pow2(9), 16);
    assert_eq!(next_pow2(15), 16);
    assert_eq!(next_pow2(16), 16);
    assert_eq!(next_pow2(17), 32);
    assert_eq!(next_pow2(1000), 1024);
  }

  #[test]
  fn test_find_size_class_exact_matches() {
    let class =
      find_size_class(QUANTUM).expect("Should find quantum size class");
    assert_eq!(*class.class_size(), QUANTUM);

    let class = find_size_class(32).expect("Should find 32 byte class");
    assert_eq!(*class.class_size(), 32);

    let class = find_size_class(1024).expect("Should find 1024 byte class");
    assert_eq!(*class.class_size(), 1024);
  }

  #[test]
  fn test_find_size_class_rounds_up() {
    let class = find_size_class(1).expect("Should find class for 1 byte");
    assert_eq!(*class.class_size(), QUANTUM);

    let class = find_size_class(17).expect("Should find class for 17 bytes");
    assert_eq!(*class.class_size(), 32);

    let class = find_size_class(100).expect("Should find class for 100 bytes");
    assert_eq!(*class.class_size(), 112);

    let class =
      find_size_class(1000).expect("Should find class for 1000 bytes");
    assert_eq!(*class.class_size(), 1024);
  }

  #[test]
  fn test_size_classes_are_sorted() {
    for i in 1..SIZE_CLASSES.len() {
      assert!(
        SIZE_CLASSES[i - 1].class_size < SIZE_CLASSES[i].class_size,
        "Size classes should be in ascending order"
      );
    }
  }

  #[test]
  fn test_size_class_alignment() {
    for class in SIZE_CLASSES {
      assert!(
        *class.element_align() >= QUANTUM,
        "Alignment should be at least QUANTUM ({}), got {}",
        QUANTUM,
        class.element_align()
      );

      assert!(
        class.element_align().is_power_of_two(),
        "Alignment {} should be power of 2",
        class.element_align()
      );
    }
  }
}
