use std::mem;

fn main() {
    println!("=== Platform Memory Alignment Analysis ===");
    println!("Word size: {} bytes", mem::size_of::<usize>());
    println!("Word alignment: {} bytes", mem::align_of::<usize>());

    const SEGMENT_SIZE: usize = 131072;
    const LARGEST_OBJECT_SIZE: usize = 65536;

    println!("Segment size: {} bytes", SEGMENT_SIZE);
    println!("Largest object size: {} bytes", LARGEST_OBJECT_SIZE);

    struct SegmentMock {
        class_ptr: *const u8,
        link_next: *mut SegmentMock,
        link_prev: *mut SegmentMock,
        link_owner: *mut u8,
        bitmap_ptr: *mut usize,
        bitmap_len: usize,
        bitmap_capacity: usize,
        user_ptr: *mut u8,
        user_len: usize,
    }

    let segment_size = mem::size_of::<SegmentMock>();
    println!("Segment struct size: {} bytes", segment_size);

    for trial in 0..10 {
        let buffer = vec![0u8; SEGMENT_SIZE];
        let buffer_addr = buffer.as_ptr() as usize;

        let after_segment = buffer_addr + segment_size;
        let bitmap_align = mem::align_of::<usize>();
        let bitmap_aligned = (after_segment + bitmap_align - 1) & !(bitmap_align - 1);
        let bitmap_offset = bitmap_aligned - after_segment;

        let after_bitmap = bitmap_aligned + mem::size_of::<usize>();
        let user_aligned = (after_bitmap + LARGEST_OBJECT_SIZE - 1) & !(LARGEST_OBJECT_SIZE - 1);
        let user_offset = user_aligned - after_bitmap;

        let total_overhead = segment_size + bitmap_offset + mem::size_of::<usize>() + user_offset;
        let user_space = SEGMENT_SIZE - total_overhead;
        let remainder = user_space % LARGEST_OBJECT_SIZE;

        println!("Trial {}: buffer@0x{:x}, bitmap_offset={}, user_offset={}, remainder={}",
                 trial, buffer_addr, bitmap_offset, user_offset, remainder);
    }

    println!("\nRun this on WSL-Ubuntu, Arch, and Ubuntu to compare alignment behavior.");
}