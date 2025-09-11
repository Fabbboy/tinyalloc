use crate::{LARGE_SC_LIMIT, MEDIUM_SC_LIMIT, MIN_ALIGN, MIN_SIZE, SIZES, SMALL_SC_LIMIT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Align(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Class(pub Size, pub Align);

const fn size_to_align(size: usize) -> usize {
    if size <= MIN_ALIGN {
        MIN_ALIGN
    } else if size <= SMALL_SC_LIMIT {
        let mut align = MIN_ALIGN;
        while align < size && align < SMALL_SC_LIMIT / 4 {
            align *= 2;
        }
        align
    } else if size <= MEDIUM_SC_LIMIT {
        let mut align = SMALL_SC_LIMIT / 4;
        while align < size && align < MEDIUM_SC_LIMIT / 8 {
            align *= 2;
        }
        align
    } else {
        let mut align = MEDIUM_SC_LIMIT / 8;
        while align < size && align * 8 <= size {
            align *= 2;
        }
        align
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
