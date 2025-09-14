use crate::mapper::Mapper;
#[cfg(unix)]
use crate::posix::PosixMapper;
#[cfg(windows)]
use crate::windows::WindowsMapper;

pub mod mapper;
pub mod posix;
pub mod region;
pub mod size;
pub mod windows;

#[cfg(unix)]
static BACKING_MAPPER: PosixMapper = PosixMapper;
#[cfg(windows)]
static BACKING_MAPPER: WindowsMapper = WindowsMapper;

#[cfg(any(unix, windows))]
pub static GLOBAL_MAPPER: &dyn Mapper = &BACKING_MAPPER;

#[derive(Debug)]
pub enum MapError {
  InvalidSize,
  OutOfMemory,
  CommitFailed,
  DecommitFailed,
  ProtectFailed,
}
