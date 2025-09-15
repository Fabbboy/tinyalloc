use std::{
  alloc::Layout,
  ptr::NonNull,
};

use tinyalloc_list::List;

use crate::{
  classes::{
    class_init,
    find_class,
  },
  config::{
    LARGE_SC_LIMIT,
    SIZES,
  },
  large::Large,
  queue::Queue,
  static_::allocate_segment,
};

pub struct Heap<'mapper> {
  classes: [Queue<'mapper>; SIZES],
  large: List<Large<'mapper>>,
}

impl<'mapper> Heap<'mapper> {
  pub fn new() -> Self {
    let classes: [Queue<'mapper>; SIZES] =
      class_init(|class| Queue::new(class));
    Self {
      classes,
      large: List::new(),
    }
  }

  pub fn allocate(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
    let size = layout.size();
    
    if size > LARGE_SC_LIMIT {
      return self.alloc_large(layout);
    }
    
    self.alloc_small(layout)
  }
  fn alloc_small(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
    let class = find_class(layout.size())?;
    let queue = &mut self.classes[class.id];
    
    if let Some(mut segment) = queue.get_available() {
      if let Some(ptr) = unsafe { segment.as_mut() }.alloc() {
        let slice = unsafe { 
          core::slice::from_raw_parts_mut(ptr.as_ptr(), class.size.0)
        };
        return NonNull::new(slice as *mut [u8]);
      }
    }
    
    let mut new_segment = allocate_segment(class).ok()?;
    queue.add_segment(new_segment);
    
    let ptr = unsafe { new_segment.as_mut() }.alloc()?;
    let slice = unsafe { 
      core::slice::from_raw_parts_mut(ptr.as_ptr(), class.size.0)
    };
    NonNull::new(slice as *mut [u8])
  }

  fn alloc_large(&mut self, _layout: Layout) -> Option<NonNull<[u8]>> {
    todo!()
  }

  pub fn deallocate(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
    todo!()
  }
}
