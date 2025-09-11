use tinyalloc::classes::CLASSES;

fn main() {
    println!("SHIFT: {}", tinyalloc::SHIFT);
    println!("SEGMENT_SHIFT: {}", tinyalloc::SEGMENT_SHIFT);
    println!("LARGE_SC_LIMIT: {}", tinyalloc::LARGE_SC_LIMIT);
    
    for (i, class) in CLASSES.iter().enumerate() {
        println!("Class {}: Size({}), Align({})", i, class.0.0, class.1.0);
        if i > 10 { break; }
    }
}