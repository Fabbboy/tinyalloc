use std::ptr::NonNull;

use thiserror::Error;

#[cfg(test)]
pub mod tests;

pub trait Item<T> {
    fn next(&self) -> Option<NonNull<T>>;
    fn prev(&self) -> Option<NonNull<T>>;
    fn set_next(&mut self, next: Option<NonNull<T>>);
    fn set_prev(&mut self, prev: Option<NonNull<T>>);
}

#[derive(Debug, Error)]
pub enum ListError {}

#[derive(Debug)]
pub struct List<T>
where
    T: Item<T>,
{
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
}

impl<T> List<T>
where
    T: Item<T>,
{
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }
}
