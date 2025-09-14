use std::ptr::NonNull;

use tinyalloc_bitmap::Bitmap;
use tinyalloc_list::{HasLink, Link};

use crate::{classes::Class, config::align_slice};

pub struct Segment<'mapper> {
    class: &'static Class,
    link: Link<Segment<'mapper>>,
    bitmap: Bitmap<'mapper, usize>,
    user: &'mapper mut [u8],
}

impl<'mapper> Segment<'mapper> {
    pub fn new(class: &'static Class, slice: &'mapper mut [u8]) -> NonNull<Self> {
        let self_size = core::mem::size_of::<Self>();
        let (segment_slice, rest) = slice.split_at_mut(self_size);

        let aligned_rest = align_slice(rest, core::mem::align_of::<usize>());
        let segmentation = class.segment::<usize>(aligned_rest);
        let bitmap = Bitmap::zero(segmentation.bitmap);

        let segment_ptr = segment_slice.as_mut_ptr() as *mut Self;
        unsafe {
            core::ptr::write(
                segment_ptr,
                Self {
                    class,
                    link: Link::new(),
                    bitmap,
                    user: segmentation.rest,
                },
            );
            NonNull::new_unchecked(segment_ptr)
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
