use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum SliceError {
    OutOfBounds { index: usize, size: usize },
    InsufficientCapacity { have: usize, need: usize },
}

#[derive(Debug)]
pub struct Slice<'data, T> {
    data: &'data mut [T],
    len: usize,
}

impl<'data, T> Slice<'data, T> {
    pub fn new(slice: &'data mut [T]) -> Self {
        Self { data: slice, len: 0 }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.data.len()
    }

    pub fn push(&mut self, value: T) -> Result<(), SliceError> {
        if self.is_full() {
            return Err(SliceError::InsufficientCapacity {
                have: self.capacity(),
                need: self.len + 1,
            });
        }

        self.data[self.len] = value;
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<()> {
        if self.is_empty() {
            None
        } else {
            self.len -= 1;
            Some(())
        }
    }

    pub fn get(&self, index: usize) -> Result<&T, SliceError> {
        if index >= self.len {
            Err(SliceError::OutOfBounds {
                index,
                size: self.len,
            })
        } else {
            Ok(&self.data[index])
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut T, SliceError> {
        if index >= self.len {
            Err(SliceError::OutOfBounds {
                index,
                size: self.len,
            })
        } else {
            Ok(&mut self.data[index])
        }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        unsafe { self.data.get_unchecked(index) }
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.data.get_unchecked_mut(index) }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data[..self.len]
    }
}

impl<'data, T> Deref for Slice<'data, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'data, T> DerefMut for Slice<'data, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_new() {
        let mut backing = [0i32; 4];
        let slice = Slice::new(&mut backing);
        
        assert_eq!(slice.len(), 0);
        assert_eq!(slice.capacity(), 4);
        assert!(slice.is_empty());
        assert!(!slice.is_full());
    }

    #[test]
    fn test_slice_push_pop() {
        let mut backing = [0i32; 3];
        let mut slice = Slice::new(&mut backing);
        
        assert!(slice.push(1).is_ok());
        assert!(slice.push(2).is_ok());
        assert_eq!(slice.len(), 2);
        
        assert_eq!(slice.pop(), Some(()));
        assert_eq!(slice.len(), 1);
        assert_eq!(slice.pop(), Some(()));
        assert_eq!(slice.len(), 0);
        assert_eq!(slice.pop(), None);
    }

    #[test]
    fn test_slice_capacity_exceeded() {
        let mut backing = [0i32; 2];
        let mut slice = Slice::new(&mut backing);
        
        assert!(slice.push(1).is_ok());
        assert!(slice.push(2).is_ok());
        assert!(slice.is_full());
        
        let result = slice.push(3);
        assert!(result.is_err());
        match result {
            Err(SliceError::InsufficientCapacity { have, need }) => {
                assert_eq!(have, 2);
                assert_eq!(need, 3);
            }
            _ => panic!("Expected InsufficientCapacity error"),
        }
    }

    #[test]
    fn test_slice_get() {
        let mut backing = [0i32; 3];
        let mut slice = Slice::new(&mut backing);
        
        slice.push(10).unwrap();
        slice.push(20).unwrap();
        
        assert_eq!(*slice.get(0).unwrap(), 10);
        assert_eq!(*slice.get(1).unwrap(), 20);
        
        let result = slice.get(2);
        assert!(result.is_err());
        match result {
            Err(SliceError::OutOfBounds { index, size }) => {
                assert_eq!(index, 2);
                assert_eq!(size, 2);
            }
            _ => panic!("Expected OutOfBounds error"),
        }
    }

    #[test]
    fn test_slice_clear() {
        let mut backing = [0i32; 3];
        let mut slice = Slice::new(&mut backing);
        
        slice.push(1).unwrap();
        slice.push(2).unwrap();
        assert_eq!(slice.len(), 2);
        
        slice.clear();
        assert_eq!(slice.len(), 0);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_slice_deref() {
        let mut backing = [0i32; 3];
        let mut slice = Slice::new(&mut backing);
        
        slice.push(1).unwrap();
        slice.push(2).unwrap();
        
        let slice_ref: &[i32] = &slice;
        assert_eq!(slice_ref.len(), 2);
        assert_eq!(slice_ref[0], 1);
        assert_eq!(slice_ref[1], 2);
    }
}