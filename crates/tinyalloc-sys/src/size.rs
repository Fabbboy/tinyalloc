use page_size::get;

#[inline]
pub fn page_size() -> usize {
  get()
}

#[inline]
pub fn page_align(size: usize) -> usize {
  let page_size = page_size();
  (size + page_size - 1) & !(page_size - 1)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_page_size_returns_valid_size() {
    let size = page_size();
    assert!(size > 0);
    assert!(size.is_power_of_two());
  }

  #[test]
  fn test_page_align_already_aligned() {
    let ps = page_size();
    assert_eq!(page_align(ps), ps);
    assert_eq!(page_align(ps * 2), ps * 2);
  }

  #[test]
  fn test_page_align_not_aligned() {
    let ps = page_size();
    assert_eq!(page_align(1), ps);
    assert_eq!(page_align(ps - 1), ps);
    assert_eq!(page_align(ps + 1), ps * 2);
  }

  #[test]
  fn test_page_align_zero() {
    assert_eq!(page_align(0), 0);
  }
}
