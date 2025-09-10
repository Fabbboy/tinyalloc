use std::sync::OnceLock;

pub fn page_align(size: usize) -> usize {
    let page = page_helper();
    size.next_multiple_of(page)
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
