use tinyalloc_config::classes::{class_init, find_class, CLASSES};
use std::alloc::Layout;

fn main() {
    let _classes = class_init(|_| ());

    let layout = Layout::from_size_align(64, 8).unwrap();
    let class = find_class(layout.size(), layout.align()).unwrap();

    println!("Layout: size={}, align={}", layout.size(), layout.align());
    println!("Mapped to class: size={}, align={}, id={}",
             class.size.0, class.align.0, class.id);

    // Show first 10 size classes
    println!("\nFirst 10 size classes:");
    for i in 0..10 {
        let c = &CLASSES[i];
        println!("  Class {}: size={}, align={}", i, c.size.0, c.align.0);
    }
}