use tinyalloc_core::vm::Mapper;

#[cfg(unix)]
use crate::posix::PosixMapper;

#[cfg(unix)]
static BACKING_MAPPER: PosixMapper = PosixMapper;

#[cfg(windows)]
static BACKING_MAPPER: WindowsMapper = WindowsMapper;

static MAPPER: &dyn Mapper = &BACKING_MAPPER;
