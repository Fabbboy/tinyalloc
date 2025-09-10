use std::ptr::NonNull;

use thiserror::Error;

pub trait Item {
    fn next(&self) -> Option<NonNull<Self>>;
    fn prev(&self) -> Option<NonNull<Self>>;
    fn set_next(&mut self, next: Option<NonNull<Self>>);
    fn set_prev(&mut self, prev: Option<NonNull<Self>>);
}

#[derive(Debug, Error)]
pub enum ListError {}
