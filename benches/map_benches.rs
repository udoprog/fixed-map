use criterion::{Bencher, Benchmark, Criterion};

fn get_bench(criterion: &mut Criterion) {
    fn fixed_map(b: &mut Bencher) {
        #[allow(unused)]
        #[derive(fixed_map::Key)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let mut map = fixed_map::Map::<_, u32>::new();
        map.insert(Key::One, 1);
        map.insert(Key::Four, 4);

        b.iter(|| map.get(&Key::Four))
    }

    fn array(b: &mut Bencher) {
        #[allow(unused)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let mut map = [None; 4];
        map[Key::One as usize] = Some(1);
        map[Key::Four as usize] = Some(4);

        b.iter(|| map[Key::Four as usize])
    }

    fn hashbrown(b: &mut Bencher) {
        #[allow(unused)]
        #[derive(PartialEq, Eq, Hash)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let mut map = hashbrown::HashMap::<_, u32>::with_capacity(4);
        map.insert(Key::One, 1);
        map.insert(Key::Four, 4);

        b.iter(|| map.get(&Key::Four))
    }

    criterion.bench(
        "get",
        Benchmark::new("fixed_map", fixed_map)
            .with_function("array", array)
            .with_function("hashbrown", hashbrown),
    );
}

fn insert_bench(criterion: &mut Criterion) {
    fn fixed_map(b: &mut Bencher) {
        #[allow(unused)]
        #[derive(fixed_map::Key)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let map = fixed_map::Map::<Key, u32>::new();

        b.iter(|| {
            let mut map = map.clone();
            map.insert(Key::Four, 4);
            map.get(&Key::Four).cloned()
        })
    }

    fn array(b: &mut Bencher) {
        #[allow(unused)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let map = [None; 4] as [Option<usize>; 4];

        b.iter(|| {
            let mut map = map.clone();
            map[Key::Four as usize] = Some(4);
            map[Key::Four as usize].as_ref().cloned()
        })
    }

    fn hashbrown(b: &mut Bencher) {
        #[allow(unused)]
        #[derive(Clone, PartialEq, Eq, Hash)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let map = hashbrown::HashMap::<_, u32>::with_capacity(4);

        b.iter(|| {
            let mut map = map.clone();
            map.insert(Key::Four, 4);
            map.get(&Key::Four).cloned()
        })
    }

    criterion.bench(
        "insert",
        Benchmark::new("fixed_map", fixed_map)
            .with_function("array", array)
            .with_function("hashbrown", hashbrown),
    );
}

fn iter_bench(criterion: &mut Criterion) {
    fn fixed_map(b: &mut Bencher) {
        #[allow(unused)]
        #[derive(fixed_map::Key)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let mut map = fixed_map::Map::<_, u32>::new();
        map.insert(Key::One, 1);
        map.insert(Key::Four, 4);

        b.iter(|| {
            let mut count = 0;
            map.iter(|_| count += 1);
            count
        })
    }

    fn array(b: &mut Bencher) {
        #[allow(unused)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let mut map = [None; 4];
        map[Key::One as usize] = Some(1);
        map[Key::Four as usize] = Some(4);

        b.iter(|| {
            let mut count = 0;

            for i in 0..4 {
                if map[i].is_some() {
                    count += 1;
                }
            }

            count
        })
    }

    fn hashbrown(b: &mut Bencher) {
        #[allow(unused)]
        #[derive(PartialEq, Eq, Hash)]
        pub enum Key {
            One,
            Two,
            Three,
            Four,
        }

        let mut map = hashbrown::HashMap::<_, u32>::with_capacity(4);
        map.insert(Key::One, 1);
        map.insert(Key::Four, 4);

        b.iter(|| map.iter().count())
    }

    criterion.bench(
        "iter",
        Benchmark::new("fixed_map", fixed_map)
            .with_function("array", array)
            .with_function("hashbrown", hashbrown),
    );
}

criterion::criterion_group! {
    name = map_group;
    config = Criterion::default();
    targets = get_bench, insert_bench, iter_bench
}
criterion::criterion_main!(map_group);
