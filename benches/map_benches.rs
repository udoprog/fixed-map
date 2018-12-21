use criterion::{Bencher, Benchmark, Criterion};

/// Macro to build a benchmark.
macro_rules! bench {
    ($id:ident, $len:expr, ($($member:ident,)*), ($($insert:ident,)*), $get:ident) => {
    fn $id(criterion: &mut Criterion) {
        fn get_fixed_map(b: &mut Bencher) {
            #[allow(unused)]
            #[derive(Clone, Copy, fixed_map::Key)]
            pub enum Key {
                $($member,)*
            }

            let mut it = 1..;
            let mut map = fixed_map::Map::<_, u32>::new();
            $(map.insert(Key::$insert, it.next().unwrap());)*

            b.iter(|| map.get(Key::$get))
        }

        fn get_array(b: &mut Bencher) {
            #[allow(unused)]
            pub enum Key {
                $($member,)*
            }

            let mut it = 1..;
            let mut map = [None; $len];
            $(map[Key::$insert as usize] = Some(it.next().unwrap());)*

            b.iter(|| map[Key::$get as usize])
        }

        fn get_hashbrown(b: &mut Bencher) {
            #[allow(unused)]
            #[derive(PartialEq, Eq, Hash)]
            pub enum Key {
                $($member,)*
            }

            let mut it = 1..;
            let mut map = hashbrown::HashMap::<_, u32>::with_capacity($len);
            $(map.insert(Key::$insert, it.next().unwrap());)*

            b.iter(|| map.get(&Key::$get))
        }

        criterion.bench(
            concat!("get_", stringify!($id)),
            Benchmark::new("fixed_map", get_fixed_map)
                .with_function("array", get_array)
                .with_function("hashbrown", get_hashbrown),
        );

        fn insert_fixed_map(b: &mut Bencher) {
            #[allow(unused)]
            #[derive(Clone, Copy, fixed_map::Key)]
            pub enum Key {
                $($member,)*
            }

            let map = fixed_map::Map::<Key, u32>::new();

            b.iter(|| {
                let mut map = map.clone();
                map.insert(Key::$get, 4);
                map.get(Key::$get).cloned()
            })
        }

        fn insert_array(b: &mut Bencher) {
            #[allow(unused)]
            pub enum Key {
                $($member,)*
            }

            let map = [None; $len] as [Option<usize>; $len];

            b.iter(|| {
                let mut map = map.clone();
                map[Key::$get as usize] = Some(4);
                map[Key::$get as usize].as_ref().cloned()
            })
        }

        fn insert_hashbrown(b: &mut Bencher) {
            #[allow(unused)]
            #[derive(Clone, PartialEq, Eq, Hash)]
            pub enum Key {
                $($member,)*
            }

            let map = hashbrown::HashMap::<_, u32>::with_capacity($len);

            b.iter(|| {
                let mut map = map.clone();
                map.insert(Key::$get, 4);
                map.get(&Key::$get).cloned()
            })
        }

        criterion.bench(
            concat!("insert_", stringify!($id)),
            Benchmark::new("fixed_map", insert_fixed_map)
                .with_function("array", insert_array)
                .with_function("hashbrown", insert_hashbrown),
        );

        fn iter_fixed_map(b: &mut Bencher) {
            #[allow(unused)]
            #[derive(Clone, Copy, fixed_map::Key)]
            pub enum Key {
                $($member,)*
            }

            let mut it = 1..;
            let mut map = fixed_map::Map::<_, u32>::new();
            $(map.insert(Key::$insert, it.next().unwrap());)*

            b.iter(|| {
                let mut count = 0;
                map.iter_fn(|_| count += 1);
                count
            })
        }

        fn iter_array(b: &mut Bencher) {
            #[allow(unused)]
            pub enum Key {
                $($member,)*
            }

            let mut it = 1..;
            let mut map = [None; $len];
            $(map[Key::$insert as usize] = Some(it.next().unwrap());)*

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

        fn iter_hashbrown(b: &mut Bencher) {
            #[allow(unused)]
            #[derive(PartialEq, Eq, Hash)]
            pub enum Key {
                $($member,)*
            }

            let mut it = 1..;
            let mut map = hashbrown::HashMap::<_, u32>::with_capacity($len);
            $(map.insert(Key::$insert, it.next().unwrap());)*

            b.iter(|| map.iter().count())
        }

        criterion.bench(
            concat!("iter_", stringify!($id)),
            Benchmark::new("fixed_map", iter_fixed_map)
                .with_function("array", iter_array)
                .with_function("hashbrown", iter_hashbrown),
        );
    }
    }
}

bench! {
    bench4,
    4,
    (T0, T1, T2, T3,),
    (T0, T3,),
    T3
}

bench! {
    bench8,
    8,
    (T0, T1, T2, T3 ,T4, T5, T6, T7,),
    (T0, T3, T6,),
    T3
}

bench! {
    bench16,
    16,
    (T0, T1, T2, T3 ,T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,),
    (T0, T3, T6, T12, T14,),
    T14
}

criterion::criterion_group! {
    name = map_group;
    config = Criterion::default();
    targets = bench4, bench8, bench16
}

criterion::criterion_main!(map_group);
