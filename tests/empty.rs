use fixed_map::{Key, Map};

#[derive(Debug, Clone, Copy, Key)]
enum Key {}

#[test]
fn empty() {
    let _ = Map::<Key, u32>::new();
}
