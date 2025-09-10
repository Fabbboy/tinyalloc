use thiserror::Error;

pub mod mapper;
pub mod posix;
pub mod region;
pub mod size;

#[derive(Debug, Error)]
pub enum MapError {
    #[error("Invalid size for memory mapping")]
    InvalidSize,
    #[error("Failed to map memory region")]
    OutOfMemory,
    #[error("Failed to commit memory region")]
    CommitFailed,
    #[error("Failed to decommit memory region")]
    DecommitFailed,
    #[error("Failed to protect memory region")]
    ProtectFailed,
}
