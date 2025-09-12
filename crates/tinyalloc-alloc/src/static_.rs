use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicPtr, Ordering},
};

use heapless::Vec;
use tinyalloc_sys::mapper::Mapper;

use crate::{
    arena::Arena,
    config::{ARENA_BATCH, ARENA_INITIAL_SIZE, ARENA_LIMIT, SEGMENT_SIZE},
};

pub type ArenaAlias = Arena<'static, dyn Mapper>;
static mut ARENAS: Vec<AtomicPtr<ArenaAlias>, ARENA_LIMIT> = Vec::new();

pub struct Manager;

impl Manager {
    fn arena_size(index: usize) -> usize {
        ARENA_INITIAL_SIZE << (index / ARENA_BATCH)
    }

    fn arena_config(index: usize) -> NonZeroUsize {
        NonZeroUsize::new(Self::arena_size(index)).unwrap()
    }
}

pub static GLOBAL_MANAGER: Manager = Manager;
