use crate::{classes::class_init, config::SIZES, queue::Queue};

pub struct Heap<'mapper> {
    classes: [Queue<'mapper>; SIZES],
}

impl<'mapper> Heap<'mapper> {
    pub fn new() -> Self {
        let classes: [Queue<'mapper>; SIZES] = class_init(|class| Queue::new(class));
        Self { classes }
    }
}
