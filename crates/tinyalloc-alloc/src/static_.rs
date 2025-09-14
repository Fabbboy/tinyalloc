use std::sync::{
  RwLock,
  atomic::{
    AtomicPtr,
    Ordering,
  },
};

use tinyalloc_array::Array;

use crate::{
  arena::{
    Arena,
    ArenaError,
  },
  config::{
    ARENA_INITIAL_SIZE,
    ARENA_LIMIT,
  },
};

use std::ptr::NonNull;

use crate::{
  classes::Class,
  segment::Segment,
};

static ARENAS: RwLock<Array<AtomicPtr<Arena<'static>>, ARENA_LIMIT>> =
  RwLock::new(Array::new());

pub fn create_arena() -> Result<NonNull<Arena<'static>>, ArenaError> {
  Arena::new(ARENA_INITIAL_SIZE)
}

pub fn add_arena(arena: NonNull<Arena<'static>>) -> Result<(), ArenaError> {
  let mut arenas = ARENAS.write().unwrap();
  let atomic_ptr = AtomicPtr::new(arena.as_ptr());
  arenas
    .push(atomic_ptr)
    .map_err(|_| ArenaError::Insufficient)?;
  Ok(())
}

pub fn get_arena_count() -> usize {
  let arenas = ARENAS.read().unwrap();
  arenas.len()
}


pub fn allocate_segment(
  class: &'static Class,
) -> Result<NonNull<Segment<'static>>, ArenaError> {
  let arenas = ARENAS.read().unwrap();

  for i in 0..arenas.len() {
    let arena_ptr = unsafe { arenas.get_unchecked(i) }.load(Ordering::Acquire);
    if !arena_ptr.is_null() {
      let arena = unsafe { &mut *arena_ptr };
      if arena.has_space() {
        if let Ok(segment) = arena.allocate(class) {
          return Ok(segment);
        }
      }
    }
  }

  drop(arenas);

  let new_arena = create_arena()?;
  add_arena(new_arena)?;

  let arena = unsafe { &mut *new_arena.as_ptr() };
  arena.allocate(class)
}

pub fn deallocate_segment(
  segment: NonNull<Segment<'static>>,
) -> Result<(), ArenaError> {
  let arenas = ARENAS.read().unwrap();
  let segment_ptr = segment.as_ptr() as *const u8;

  for i in 0..arenas.len() {
    let arena_ptr = unsafe { arenas.get_unchecked(i) }.load(Ordering::Acquire);
    if !arena_ptr.is_null() {
      let arena = unsafe { &mut *arena_ptr };
      let arena_start = arena.user_start();
      let arena_end = unsafe { arena_start.add(arena.user_len()) };

      if segment_ptr >= arena_start && segment_ptr < arena_end {
        return arena.deallocate(segment);
      }
    }
  }

  Err(ArenaError::Insufficient)
}
