
use tinyalloc_list::{Link, HasLink};

pub struct Segment<'mapper> {
    link: Link<Segment<'mapper>>,
    data: &'mapper [u8],
}

impl<'mapper> Default for Segment<'mapper> {
    fn default() -> Self {
        Self {
            link: Default::default(),
            data: &[],
        }
    }
}

impl<'mapper> Segment<'mapper> {
    pub fn new(data: &'mapper [u8]) -> Self {
        Self {
            data,
            ..Default::default()
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
