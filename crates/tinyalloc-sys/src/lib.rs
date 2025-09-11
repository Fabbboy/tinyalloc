
pub mod mapper;
pub mod posix;
pub mod region;
pub mod size;

#[derive(Debug)]
pub enum MapError {
    InvalidSize,
    OutOfMemory,
    CommitFailed,
    DecommitFailed,
    ProtectFailed,
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::InvalidSize => write!(f, "Invalid size for memory mapping"),
            MapError::OutOfMemory => write!(f, "Failed to map memory region"),
            MapError::CommitFailed => write!(f, "Failed to commit memory region"),
            MapError::DecommitFailed => write!(f, "Failed to decommit memory region"),
            MapError::ProtectFailed => write!(f, "Failed to protect memory region"),
        }
    }
}

impl std::error::Error for MapError {}
