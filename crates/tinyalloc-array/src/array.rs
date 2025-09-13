use core::{mem::MaybeUninit, ops::{Deref, DerefMut}, slice};

#[derive(Debug)]
pub enum ArrayError {
    OutOfBounds { index: usize, size: usize },
    InsufficientCapacity { have: usize, need: usize },
}

#[derive(Debug)]
pub struct Array<T, const SIZE: usize> {
    data: [MaybeUninit<T>; SIZE],
    len: usize,
}

impl<T, const SIZE: usize> Array<T, SIZE> {
    pub const fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        SIZE
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == SIZE
    }

    pub fn push(&mut self, value: T) -> Result<(), ArrayError> {
        if self.is_full() {
            return Err(ArrayError::InsufficientCapacity {
                have: SIZE,
                need: SIZE + 1,
            });
        }

        unsafe {
            self.data.get_unchecked_mut(self.len).write(value);
        }
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.data.get_unchecked(self.len).assume_init_read() })
        }
    }

    pub fn get(&self, index: usize) -> Result<&T, ArrayError> {
        if index >= self.len {
            Err(ArrayError::OutOfBounds {
                index,
                size: self.len,
            })
        } else {
            Ok(unsafe { self.data.get_unchecked(index).assume_init_ref() })
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut T, ArrayError> {
        if index >= self.len {
            Err(ArrayError::OutOfBounds {
                index,
                size: self.len,
            })
        } else {
            Ok(unsafe { self.data.get_unchecked_mut(index).assume_init_mut() })
        }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        unsafe { self.data.get_unchecked(index).assume_init_ref() }
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.data.get_unchecked_mut(index).assume_init_mut() }
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
    }
}

impl<T, const SIZE: usize> Default for Array<T, SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SIZE: usize> Deref for Array<T, SIZE> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const SIZE: usize> DerefMut for Array<T, SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const SIZE: usize> Drop for Array<T, SIZE> {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_new() {
        let arr: Array<i32, 4> = Array::new();
        
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.capacity(), 4);
        assert!(arr.is_empty());
        assert!(!arr.is_full());
    }

    #[test]
    fn test_array_push_pop() {
        let mut arr: Array<i32, 3> = Array::new();
        
        assert!(arr.push(1).is_ok());
        assert!(arr.push(2).is_ok());
        assert_eq!(arr.len(), 2);
        assert!(!arr.is_empty());
        
        assert_eq!(arr.pop(), Some(2));
        assert_eq!(arr.pop(), Some(1));
        assert_eq!(arr.len(), 0);
        assert!(arr.is_empty());
        assert_eq!(arr.pop(), None);
    }

    #[test]
    fn test_array_capacity_exceeded() {
        let mut arr: Array<i32, 2> = Array::new();
        
        assert!(arr.push(1).is_ok());
        assert!(arr.push(2).is_ok());
        assert!(arr.is_full());
        
        let result = arr.push(3);
        assert!(result.is_err());
        match result {
            Err(ArrayError::InsufficientCapacity { have, need }) => {
                assert_eq!(have, 2);
                assert_eq!(need, 3);
            }
            _ => panic!("Expected InsufficientCapacity error"),
        }
    }

    #[test]
    fn test_array_get() {
        let mut arr: Array<i32, 4> = Array::new();
        
        arr.push(10).unwrap();
        arr.push(20).unwrap();
        
        assert_eq!(*arr.get(0).unwrap(), 10);
        assert_eq!(*arr.get(1).unwrap(), 20);
        
        let result = arr.get(2);
        assert!(result.is_err());
        match result {
            Err(ArrayError::OutOfBounds { index, size }) => {
                assert_eq!(index, 2);
                assert_eq!(size, 2);
            }
            _ => panic!("Expected OutOfBounds error"),
        }
    }

    #[test]
    fn test_array_get_mut() {
        let mut arr: Array<i32, 4> = Array::new();
        
        arr.push(10).unwrap();
        arr.push(20).unwrap();
        
        *arr.get_mut(0).unwrap() = 100;
        assert_eq!(*arr.get(0).unwrap(), 100);
        
        let result = arr.get_mut(2);
        assert!(result.is_err());
        match result {
            Err(ArrayError::OutOfBounds { index, size }) => {
                assert_eq!(index, 2);
                assert_eq!(size, 2);
            }
            _ => panic!("Expected OutOfBounds error"),
        }
    }

    #[test]
    fn test_array_clear() {
        let mut arr: Array<i32, 4> = Array::new();
        
        arr.push(1).unwrap();
        arr.push(2).unwrap();
        arr.push(3).unwrap();
        assert_eq!(arr.len(), 3);
        
        arr.clear();
        assert_eq!(arr.len(), 0);
        assert!(arr.is_empty());
    }

    #[test]
    fn test_array_deref() {
        let mut arr: Array<i32, 4> = Array::new();
        
        arr.push(1).unwrap();
        arr.push(2).unwrap();
        arr.push(3).unwrap();
        
        let slice_ref: &[i32] = &arr;
        assert_eq!(slice_ref.len(), 3);
        assert_eq!(slice_ref[0], 1);
        assert_eq!(slice_ref[1], 2);
        assert_eq!(slice_ref[2], 3);
    }

    #[test]
    fn test_array_as_slice() {
        let mut arr: Array<i32, 4> = Array::new();
        
        arr.push(10).unwrap();
        arr.push(20).unwrap();
        
        let s = arr.as_slice();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0], 10);
        assert_eq!(s[1], 20);
    }

    #[test]
    fn test_array_as_mut_slice() {
        let mut arr: Array<i32, 4> = Array::new();
        
        arr.push(10).unwrap();
        arr.push(20).unwrap();
        
        let s = arr.as_mut_slice();
        s[0] = 100;
        assert_eq!(*arr.get(0).unwrap(), 100);
    }

    #[test]
    fn test_array_unsafe_get() {
        let mut arr: Array<i32, 4> = Array::new();
        
        arr.push(42).unwrap();
        
        unsafe {
            assert_eq!(*arr.get_unchecked(0), 42);
            *arr.get_unchecked_mut(0) = 84;
            assert_eq!(*arr.get_unchecked(0), 84);
        }
    }

    #[test]
    fn test_array_default() {
        let arr: Array<i32, 4> = Array::default();
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.capacity(), 4);
    }

    #[test]
    fn test_array_zero_capacity() {
        let mut arr: Array<i32, 0> = Array::new();
        
        assert_eq!(arr.capacity(), 0);
        assert!(arr.is_empty());
        assert!(arr.is_full());
        
        let result = arr.push(1);
        assert!(result.is_err());
        assert_eq!(arr.pop(), None);
    }
}