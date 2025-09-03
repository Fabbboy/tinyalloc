use page_size::get;

#[inline]
pub fn page_size() -> usize {
  get()
}

pub fn page_align(size: usize) -> usize {
  let page_size = page_size();
  (size + page_size - 1) & !(page_size - 1)
}
