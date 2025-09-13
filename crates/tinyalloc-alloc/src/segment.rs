use tinyalloc_array::slice::Slice;
use tinyalloc_bitmap::{Bitmap, BitmapError};
use tinyalloc_list::{HasLink, Link};

use crate::classes::Class;

pub struct Segment<'mapper> {
    class: Class,
    link: Link<Segment<'mapper>>,
    bitmap: Bitmap<usize, 1>,
    slice: Slice<'mapper, u8>,
}

impl<'mapper> Segment<'mapper> {
    pub fn new(class: Class, slice: &'mapper mut [u8]) -> Self {
        let slice = Slice::new(slice);
        Segment {
            class,
            link: Link::new(),
            bitmap: Bitmap::default(),
            slice,
        }
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
