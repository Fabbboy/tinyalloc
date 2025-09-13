/*use std::{
    num::NonZeroUsize,
    sync::{RwLock, atomic::AtomicPtr},
};

use tinyalloc_sys::mapper::Mapper;

use crate::{
    arena::Arena,
    config::{ARENA_BATCH, ARENA_INITIAL_SIZE, ARENA_LIMIT},
};

pub type ArenaAlias = Arena<'static, dyn Mapper>;
//static mut ARENAS: RwLock<Vec<AtomicPtr<ArenaAlias>, ARENA_LIMIT>> = RwLock::new(Vec::new());
static ARENAS: RwLock<[Option<AtomicPtr<ArenaAlias>>; ARENA_LIMIT]> =
    RwLock::new([(); ARENA_LIMIT].map(|_| None));

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
*/