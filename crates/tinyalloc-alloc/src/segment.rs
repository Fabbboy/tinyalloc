use std::ptr::NonNull;

use tinyalloc_bitmap::Bitmap;
use tinyalloc_list::{HasLink, Link};

use crate::classes::Class;

pub struct Segment<'mapper> {
    class: &'static Class,
    link: Link<Segment<'mapper>>,
    bitmap: Bitmap<'mapper, usize>,
    user: &'mapper mut [u8],
}

impl<'mapper> Segment<'mapper> {
    pub fn new(class: &'static Class, slice: &'mapper mut [u8]) -> NonNull<Self> {
        
    }
}

impl<'mapper> HasLink<Segment<'mapper>> for Segment<'mapper> {
    fn link(&self) -> &Link<Segment<'mapper>> {
        &self.link
    }

    fn link_mut(&mut self) -> &mut Link<Segment<'mapper>> {
        &mut self.link
    }
}
