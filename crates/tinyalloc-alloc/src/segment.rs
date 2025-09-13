use tinyalloc_bitmap::{Bitmap, BitmapError};
use tinyalloc_list::{HasLink, Link};
use tinyvec::SliceVec;

use crate::classes::Class;

pub struct Segment<'mapper> {
    class: Class,
    link: Link<Segment<'mapper>>,
    bitmap: Bitmap<usize, 1>,
    vec: SliceVec<'mapper, u8>,
}

impl<'mapper> Segment<'mapper> {
    pub fn new(class: Class, slice: &'mapper mut [u8]) -> Self {
        let vec = SliceVec::from_slice_len(slice, 0);
        Segment {
            class,
            link: Link::new(),
            bitmap: Bitmap::default(),
            vec,
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
