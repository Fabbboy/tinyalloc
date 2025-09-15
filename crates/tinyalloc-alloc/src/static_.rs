use std::sync::{
  RwLock,
  atomic::{
    AtomicPtr,
    AtomicUsize,
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
    ARENA_GROWTH,
    ARENA_INITIAL_SIZE,
    ARENA_LIMIT,
    ARENA_STEP,
  },
};

use std::ptr::NonNull;

use crate::{
  classes::Class,
  segment::Segment,
};

static ARENAS: RwLock<Array<AtomicPtr<Arena<'static>>, ARENA_LIMIT>> =
  RwLock::new(Array::new());
static NEXT_ARENA_SIZE: AtomicUsize = AtomicUsize::new(ARENA_INITIAL_SIZE);

fn create_arena() -> Result<NonNull<Arena<'static>>, ArenaError> {
  let size = NEXT_ARENA_SIZE.load(Ordering::Relaxed);
  let arena = Arena::new(size)?;
  Ok(arena)
}

fn add_arena(arena: NonNull<Arena<'static>>) -> Result<(), ArenaError> {
  let mut arenas = ARENAS.write().unwrap();
  let arena_count = arenas.len();

  if arena_count > 0 && arena_count % ARENA_STEP == 0 {
    let current_size = NEXT_ARENA_SIZE.load(Ordering::Relaxed);
    let next_size = current_size.checked_mul(ARENA_GROWTH).unwrap_or(current_size);
    NEXT_ARENA_SIZE.store(next_size, Ordering::Relaxed);
  }

  let atomic_ptr = AtomicPtr::new(arena.as_ptr());
  arenas
    .push(atomic_ptr)
    .map_err(|_| ArenaError::Insufficient)?;
  Ok(())
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
