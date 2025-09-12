use std::sync::OnceLock;

use std::ptr::NonNull;

pub fn page_align(size: usize) -> usize {
    let page = page_helper();
    size.next_multiple_of(page)
}

#[inline]
pub fn page_align_ptr(ptr: *mut u8) -> *mut u8 {
    let page = page_size();
    let addr = ptr as usize;
    let aligned_addr = addr & !(page - 1);
    aligned_addr as *mut u8
}

#[inline]
pub fn page_align_slice(slice: NonNull<[u8]>) -> NonNull<[u8]> {
    let page = page_size();
    let start_addr = slice.as_ptr() as *const u8 as usize;
    let end_addr = start_addr + slice.len();
    
    let aligned_start = start_addr & !(page - 1);
    let aligned_end = (end_addr + page - 1) & !(page - 1);
    
    let aligned_ptr = aligned_start as *mut u8;
    let aligned_size = aligned_end - aligned_start;
    
    unsafe {
        let slice = std::slice::from_raw_parts_mut(aligned_ptr, aligned_size);
        NonNull::new(slice).unwrap()
    }
}

pub fn page_size() -> usize {
    page_helper()
}

#[cfg(unix)]
fn page_helper() -> usize {
    static PAGE_SIZE: OnceLock<usize> = OnceLock::new();
    *PAGE_SIZE.get_or_init(unix::get)
}

#[cfg(not(unix))]
fn page_helper() -> usize {
    4096
}

#[cfg(unix)]
mod unix {
    #[inline]
    pub fn get() -> usize {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }
}
