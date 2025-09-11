use crate::SIZES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Align(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Class(pub Size, pub Align);

const fn classes() -> [Class; SIZES] {
    let mut classes = [Class(Size(0), Align(0)); SIZES];
    let mut i = 0;
    while i < SIZES {
        let align = 1 << (i / 2);
        let size = if i % 2 == 0 { align * 3 / 2 } else { align * 2 };
        classes[i] = Class(Size(size), Align(align));
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
