use fixed_map::{Key, Map};

#[derive(Debug, Clone, Copy, Key)]
enum MyKey {}

#[test]
fn empty() {
    let _ = Map::<MyKey, u32>::new();
}
