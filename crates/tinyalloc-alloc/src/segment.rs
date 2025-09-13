use tinyalloc_array::slice::Slice;
use tinyalloc_bitmap::Bitmap;
use tinyalloc_list::{HasLink, Link};

use crate::classes::Class;

pub struct Segment<'mapper> {
    class: &'static Class,
    link: Link<Segment<'mapper>>,
    bitmap: Bitmap<'mapper, usize>,
    slice: Slice<'mapper, u8>,
}

impl<'mapper> Segment<'mapper> {
    pub fn new(class: &'static Class, slice: &'mapper mut [u8]) -> Self {
        let slice = Slice::from_slice(slice, 0);
        let bitmap_store = Slice::from_slice(&mut [], 0);

        Segment {
            class,
            link: Link::new(),
            bitmap: Bitmap::zero(bitmap_store),
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
