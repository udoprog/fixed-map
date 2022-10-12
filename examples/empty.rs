use fixed_map::{Key, Map};

#[derive(Debug, Clone, Copy, Key)]
enum Key {}

fn main() {
    let _ = Map::<Key, u32>::new();
}
