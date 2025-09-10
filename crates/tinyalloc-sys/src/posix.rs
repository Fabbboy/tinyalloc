#[cfg(unix)]
use std::ptr::NonNull;

use crate::MapError;
use crate::mapper::Mapper;
#[cfg(unix)]
use crate::mapper::Protection;
use crate::size::page_align;
use anyhow::Result;
#[cfg(unix)]
use enumset::EnumSet;

#[cfg(unix)]
mod unix {
    pub const NULL: *mut libc::c_void = std::ptr::null_mut();
    pub const TRASH: i32 = -1;

    pub const PERM_WRITE: i32 = libc::PROT_WRITE;
    pub const PERM_READ: i32 = libc::PROT_READ;

    pub const PERM_RW: i32 = PERM_READ | PERM_WRITE;
    pub const PERM_NONE: i32 = libc::PROT_NONE;

    pub const DONTNEED: i32 = libc::MADV_DONTNEED;

    pub const MAP_FLAGS: i32 = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
}

#[derive(Clone)]
pub struct PosixMapper;

#[cfg(unix)]
impl PosixMapper {
    fn to_prot(prot: EnumSet<Protection>) -> i32 {
        let rf = prot.contains(Protection::Read);
        let wf = prot.contains(Protection::Write);
        match (rf, wf) {
            (true, true) => unix::PERM_RW,
            (true, false) => unix::PERM_READ,
            (false, true) => unix::PERM_WRITE,
            (false, false) => unix::PERM_NONE,
        }
    }
}

#[cfg(unix)]
impl Mapper for PosixMapper {
    fn map(size: usize) -> Result<NonNull<[u8]>> {
        if size == 0 {
            return Err(MapError::InvalidSize.into());
        }
        let aligned_size = page_align(size);
        let ptr = unsafe {
            libc::mmap(
                unix::NULL,
                aligned_size,
                unix::PERM_NONE,
                unix::MAP_FLAGS,
                unix::TRASH,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(MapError::OutOfMemory.into());
        }

        let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, aligned_size) };
        Ok(NonNull::new(slice).unwrap())
    }

    fn unmap(ptr: NonNull<[u8]>) {
        let size = ptr.len();
        unsafe { libc::munmap(ptr.as_ptr() as *mut libc::c_void, size) };
    }

    fn commit(ptr: NonNull<[u8]>) -> Result<()> {
        Self::protect(ptr, EnumSet::all())?;
        Ok(())
    }

    fn decommit(ptr: NonNull<[u8]>) -> Result<()> {
        let cptr = Self::cptr(ptr.as_ptr() as *mut u8);
        let res = unsafe { libc::madvise(cptr, ptr.len(), unix::DONTNEED) };
        if res != 0 {
            return Err(MapError::DecommitFailed.into());
        }
        Self::protect(ptr, EnumSet::empty())?;
        Ok(())
    }

    fn protect(ptr: NonNull<[u8]>, prot: EnumSet<Protection>) -> Result<()> {
        let prot_flags = Self::to_prot(prot);
        let cptr = Self::cptr(ptr.as_ptr() as *mut u8);
        let res = unsafe { libc::mprotect(cptr, ptr.len(), prot_flags) };
        if res != 0 {
            return Err(MapError::ProtectFailed.into());
        }
        Ok(())
    }
}

#[cfg(not(unix))]
impl Mapper for PosixMapper {}

#[cfg(all(unix, test))]
mod tests {
    use enumset::EnumSet;

    use crate::{
        mapper::{Mapper, Protection},
        posix::{PosixMapper, unix},
    };

    #[test]
    fn test_to_prot_all_combinations() {
        assert_eq!(PosixMapper::to_prot(EnumSet::empty()), unix::PERM_NONE);
        assert_eq!(
            PosixMapper::to_prot(Protection::Read.into()),
            unix::PERM_READ
        );
        assert_eq!(
            PosixMapper::to_prot(Protection::Write.into()),
            unix::PERM_WRITE
        );
        assert_eq!(PosixMapper::to_prot(EnumSet::all()), unix::PERM_RW);
    }

    #[test]
    fn test_map_and_unmap() {
        let result = PosixMapper::map(4096);
        assert!(result.is_ok());

        let ptr = result.unwrap();
        assert!(ptr.len() >= 4096);

        PosixMapper::unmap(ptr);
    }

    #[test]
    fn test_map_zero_size() {
        let result = PosixMapper::map(0);
        assert!(result.is_err());

        if let Err(e) = result {
            let map_error = e.downcast_ref::<crate::MapError>();
            assert!(matches!(map_error, Some(crate::MapError::InvalidSize)));
        }
    }

    #[test]
    fn test_commit_and_protect() {
        let ptr = PosixMapper::map(4096).unwrap();

        let commit_result = PosixMapper::commit(ptr);
        assert!(commit_result.is_ok());

        let protect_result = PosixMapper::protect(ptr, Protection::Read.into());
        assert!(protect_result.is_ok());

        PosixMapper::unmap(ptr);
    }

    #[test]
    fn test_protect_with_different_permissions() {
        let ptr = PosixMapper::map(4096).unwrap();

        assert!(PosixMapper::protect(ptr, EnumSet::empty()).is_ok());
        assert!(PosixMapper::protect(ptr, Protection::Read.into()).is_ok());
        assert!(PosixMapper::protect(ptr, Protection::Write.into()).is_ok());
        assert!(PosixMapper::protect(ptr, EnumSet::all()).is_ok());

        PosixMapper::unmap(ptr);
    }

    #[test]
    fn test_decommit() {
        let ptr = PosixMapper::map(4096).unwrap();
        PosixMapper::commit(ptr).unwrap();

        let decommit_result = PosixMapper::decommit(ptr);
        assert!(decommit_result.is_ok());
        PosixMapper::unmap(ptr);
    }

    #[test]
    fn test_large_allocation() {
        let size = 1024 * 1024;
        let ptr = PosixMapper::map(size).unwrap();
        assert!(ptr.len() >= size);

        PosixMapper::commit(ptr).unwrap();
        PosixMapper::protect(ptr, EnumSet::all()).unwrap();

        PosixMapper::unmap(ptr);
    }
}
