use std::cell::OnceCell;

use heapless::Vec;
use tinyalloc_bitmap::Bitmap;
use tinyalloc_list::{HasLink, Link};

use crate::config::SEGMENT_SIZE;

#[derive(Default)]
pub struct Segment<'mapper> {
    link: Link<Segment<'mapper>>,
    bitmap: OnceCell<Bitmap<'mapper, usize>>, // we need concrete types dont want to carve manually
    data: Vec<u8, SEGMENT_SIZE>, // not sure abt this one would be nice honestly simplifies our lvies but needs some unsafe
}

impl<'mapper> Segment<'mapper> {
    pub fn new(bitmap: Bitmap<'mapper, usize>) -> Self {
        Self {
            link: Default::default(),
            bitmap,
            data: Vec::new(),
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
