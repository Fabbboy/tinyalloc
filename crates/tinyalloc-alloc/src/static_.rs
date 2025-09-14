use std::sync::{
  RwLock,
  atomic::AtomicPtr,
};

use tinyalloc_array::Array;

use crate::{
  arena::Arena,
  config::ARENA_LIMIT,
};

pub type ArenaAlias = Arena<'static>;
static ARENAS: RwLock<Array<AtomicPtr<ArenaAlias>, ARENA_LIMIT>> =
  RwLock::new(Array::new());
