//! These two should expand to roughly the same implementation.

macro_rules! expand {
    ($len:expr, ($($member:ident),*), $get:ident) => {
        #[allow(unused)]
        #[derive(Clone, Copy, fixed_map::Key)]
        pub enum FixedKey {
            $($member,)*
        }

        #[no_mangle]
        #[inline(never)]
        pub fn test_fixed(map: &fixed_map::Map<FixedKey, u32>) -> Option<u32> {
            map.get(FixedKey::$get).cloned()
        }

        #[allow(unused)]
        #[repr(usize)]
        pub enum ArrayKey {
            $($member,)*
        }

        #[no_mangle]
        #[inline(never)]
        pub fn test_array(map: &[Option<u32>; $len]) -> Option<u32> {
            map[ArrayKey::$get as usize].as_ref().cloned()
        }
    }
}

expand! {
    16,
    (T00, T01, T02, T03, T04, T05, T06, T07, T8, T9, T10, T11, T12, T13, T14, T15),
    T14
}

fn main() {
    let mut map = fixed_map::Map::new();
    map.insert(FixedKey::T10, 42u32);

    let mut array = [None; 16];
    array[ArrayKey::T10 as usize] = Some(42u32);

    println!("Fixed: {:?}", test_fixed(&map));
    println!("Array: {:?}", test_array(&array));
}
