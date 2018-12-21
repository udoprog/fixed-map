use criterion::{Bencher, Benchmark, Criterion};

/// Macro to build a benchmark.
macro_rules! benches {
    (
    $({
        $len:expr, ($($member:ident,)*), ($($insert:ident,)*), $get:ident
    };)*
    ) => {fn benches(criterion: &mut Criterion) {
    $(
        criterion.bench(
            "fixed",
            Benchmark::new(concat!("get", stringify!($len)), |b: &mut Bencher| {
                #[allow(unused)]
                #[derive(Clone, Copy, fixed_map::Key)]
                pub enum Key {
                    $($member,)*
                }

                let mut it = 1..;
                let mut map = fixed_map::Map::<_, u32>::new();
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| map.get(Key::$get))
            }),
        );
    )*

    $(
        criterion.bench(
            "array",
            Benchmark::new(concat!("get", stringify!($len)), |b: &mut Bencher| {
                #[allow(unused)]
                pub enum Key {
                    $($member,)*
                }

                let mut it = 1..;
                let mut map = [None; $len];
                $(map[Key::$insert as usize] = Some(it.next().unwrap());)*

                b.iter(|| map[Key::$get as usize])
            }),
        );
    )*

    $(
        criterion.bench(
            "hashbrown",
            Benchmark::new(concat!("get", stringify!($len)), |b: &mut Bencher| {
                #[allow(unused)]
                #[derive(PartialEq, Eq, Hash)]
                pub enum Key {
                    $($member,)*
                }

                let mut it = 1..;
                let mut map = hashbrown::HashMap::<_, u32>::with_capacity($len);
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| map.get(&Key::$get))
            }),
        );
    )*

    $(
        criterion.bench(
            "fixed",
            Benchmark::new(concat!("insert", stringify!($len)), |b: &mut Bencher| {
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
            }),
        );
    )*

    $(
        criterion.bench(
            "array",
            Benchmark::new(concat!("insert", stringify!($len)), |b: &mut Bencher| {
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
            }),
        );
    )*

    $(
        criterion.bench(
            "hashbrown",
            Benchmark::new(concat!("insert", stringify!($len)), |b: &mut Bencher| {
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
            }),
        );
    )*

    $(
        criterion.bench(
            "fixed",
            Benchmark::new(concat!("iter", stringify!($len)), |b: &mut Bencher| {
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
            }),
        );
    )*

    $(
        criterion.bench(
            "array",
            Benchmark::new(concat!("iter", stringify!($len)), |b: &mut Bencher| {
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
            }),
        );
    )*

    $(
        criterion.bench(
            "hashbrown",
            Benchmark::new(concat!("iter", stringify!($len)), |b: &mut Bencher| {
                #[allow(unused)]
                #[derive(PartialEq, Eq, Hash)]
                pub enum Key {
                    $($member,)*
                }

                let mut it = 1..;
                let mut map = hashbrown::HashMap::<_, u32>::with_capacity($len);
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| map.iter().count())
            }),
        );
    )*
    }}
}

benches! {
    {
        4,
        (T0, T1, T2, T3,),
        (T0, T3,),
        T3
    };

    {
        8,
        (T0, T1, T2, T3 ,T4, T5, T6, T7,),
        (T0, T3, T6,),
        T3
    };

    {
        16,
        (T0, T1, T2, T3 ,T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,),
        (T0, T3, T6, T12, T14,),
        T14
    };
}

criterion::criterion_group! {
    name = map_group;
    config = Criterion::default();
    targets = benches
}

criterion::criterion_main!(map_group);
