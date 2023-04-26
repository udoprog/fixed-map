use fixed_map::{Key, Map};

#[derive(Clone, Copy, Key)]
enum Part {
    One,
    Two,
}

#[test]
fn simple() {
    let mut map: Map<Part, i32> = Map::new();

    assert_eq!(map.entry(Part::Two).or_default(), &0);
    assert_eq!(
        map.entry(Part::One).and_modify(|x| *x += 1).or_insert(12),
        &12
    );
    assert_eq!(
        map.entry(Part::One).and_modify(|x| *x += 1).or_insert(12),
        &13
    );
}

#[test]
fn other() {
    use fixed_map::{Key, Map};

    #[derive(Clone, Copy, Key)]
    enum MyKey {
        Even,
        Odd,
    }

    let mut map: Map<MyKey, u32> = Map::new();

    for n in [3, 45, 3, 23, 2, 10, 59, 11, 51, 70] {
        map.entry(if n % 2 == 0 { MyKey::Even } else { MyKey::Odd })
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }

    assert_eq!(map.get(MyKey::Even), Some(&3));
    assert_eq!(map.get(MyKey::Odd), Some(&7));
}

#[test]
fn composite() {
    use fixed_map::{Key, Map};

    #[derive(Clone, Copy, Key)]
    enum MyKey {
        First(bool),
        Second,
    }

    let mut map: Map<MyKey, Vec<i32>> = Map::new();

    map.entry(MyKey::First(true)).or_default().push(1);
    map.entry(MyKey::Second)
        .or_insert_with(|| vec![2; 8])
        .truncate(4);

    assert_eq!(map.get(MyKey::First(true)), Some(&vec![1]));
    assert_eq!(map.get(MyKey::Second), Some(&vec![2; 4]));
}

#[cfg(feature = "hashbrown")]
#[test]
fn compound() {
    #[derive(Clone, Copy, Key)]
    enum MyKey {
        Simple,
        Composite(Part),
        String(&'static str),
        Number(u32),
        Singleton(()),
    }

    let mut map: Map<MyKey, i32> = Map::new();

    map.insert(MyKey::Composite(Part::One), 1);
    assert_eq!(map.entry(MyKey::Composite(Part::Two)).or_default(), &0);
    assert_eq!(
        map.entry(MyKey::Composite(Part::One))
            .and_modify(|x| *x += 1)
            .or_insert(12),
        &2
    );
}
