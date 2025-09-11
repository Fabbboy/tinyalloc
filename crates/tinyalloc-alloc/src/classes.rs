use crate::SIZES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Align(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Class(pub Size, pub Align);

const fn classes() -> [Class; SIZES] {
    let mut classes = [Class(Size(0), Align(0)); SIZES];
    
    classes
}

pub static CLASSES: [Class; SIZES] = classes();
