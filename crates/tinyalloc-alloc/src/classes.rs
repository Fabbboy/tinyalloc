use crate::config::{
    LARGE_ALIGN_RATIO, LARGE_SC_LIMIT, MEDIUM_ALIGN_LIMIT, MEDIUM_SC_LIMIT, MIN_ALIGN, MIN_SIZE,
    SIZES, SMALL_ALIGN_LIMIT, SMALL_SC_LIMIT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Align(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Class(pub Size, pub Align);

const fn size_to_align(size: usize) -> usize {
    if size <= SMALL_ALIGN_LIMIT {
        MIN_ALIGN
    } else if size <= MEDIUM_ALIGN_LIMIT {
        SMALL_ALIGN_LIMIT
    } else if size <= LARGE_SC_LIMIT {
        MEDIUM_ALIGN_LIMIT
    } else {
        size / LARGE_ALIGN_RATIO
    }
}

const fn classes() -> [Class; SIZES] {
    let mut classes = [Class(Size(0), Align(0)); SIZES];
    let mut i = 0;
    let mut size = MIN_SIZE;

    while i < SIZES {
        let align = size_to_align(size);
        classes[i] = Class(Size(size), Align(align));

        if size < SMALL_SC_LIMIT {
            size += align;
        } else if size < MEDIUM_SC_LIMIT {
            size += align * 2;
        } else if size < LARGE_SC_LIMIT {
            size += align * 4;
        } else {
            size *= 2;
        }

        i += 1;
    }
    classes
}

pub static CLASSES: [Class; SIZES] = classes();

pub const fn find_class(size: usize) -> Option<&'static Class> {
    let mut i = 0;
    while i < SIZES {
        let class = &CLASSES[i];
        if size <= class.0.0 {
            return Some(class);
        }
        i += 1;
    }
    None
}
