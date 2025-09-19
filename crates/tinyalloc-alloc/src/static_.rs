use std::sync::atomic::{
  AtomicPtr,
  AtomicUsize,
  Ordering,
};

use spin::RwLock;
use tinyalloc_array::Array;
use tinyalloc_config::{classes::Class, config::{ARENA_GROWTH, ARENA_INITIAL_SIZE, ARENA_LIMIT, ARENA_STEP, SEGMENT_SIZE}};

use crate::{
  arena::{
    Arena,
    ArenaError,
  },
 
};

use std::ptr::NonNull;

use crate::{ 
  segment::Segment,
};

static ARENAS: RwLock<Array<AtomicPtr<Arena>, ARENA_LIMIT>> =
  RwLock::new(Array::new());
static NEXT_ARENA_SIZE: AtomicUsize = AtomicUsize::new(ARENA_INITIAL_SIZE);

fn create_arena() -> Result<NonNull<Arena>, ArenaError> {
  let size = NEXT_ARENA_SIZE.load(Ordering::Relaxed);
  let arena = Arena::new(size)?;
  Ok(arena)
}

fn add_arena(arena: NonNull<Arena>) -> Result<(), ArenaError> {
  let mut arenas = ARENAS.write();
  let arena_count = arenas.len();

  if arena_count > 0 && arena_count % ARENA_STEP == 0 {
    let current_size = NEXT_ARENA_SIZE.load(Ordering::Relaxed);
    let next_size = current_size
      .checked_mul(ARENA_GROWTH)
      .unwrap_or(current_size);
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
) -> Result<NonNull<Segment>, ArenaError> {
  let arenas = ARENAS.read();

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

pub fn deallocate_segment(segment: NonNull<Segment>) -> Result<(), ArenaError> {
  let arenas = ARENAS.read();
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

pub fn segment_from_ptr(ptr: NonNull<u8>) -> Option<NonNull<Segment>> {
  let arenas = ARENAS.read();
  let addr = ptr.as_ptr() as usize;

  for i in 0..arenas.len() {
    let arena_ptr = unsafe { arenas.get_unchecked(i) }.load(Ordering::Acquire);
    if arena_ptr.is_null() {
      continue;
    }

    let arena = unsafe { &*arena_ptr };
    let start = arena.user_start() as usize;
    let end = start.checked_add(arena.user_len())?;
    if addr < start || addr >= end {
      continue;
    }

    let offset = addr - start;
    let segment_index = offset / SEGMENT_SIZE;
    let segment_base = start + (segment_index * SEGMENT_SIZE);
    let segment_ptr = segment_base as *mut Segment;
    if let Some(segment_nn) = NonNull::new(segment_ptr) {
      if unsafe { segment_nn.as_ref() }.contains_ptr(ptr) {
        return Some(segment_nn);
      }
    }
  }

  None
}
