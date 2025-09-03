#[derive(Debug)]
pub enum BitmapError {
  InsufficientSize { have: usize, need: usize },
  OutOfBounds { index: usize, size: usize },
  OutOfSpace { size: usize },
}
