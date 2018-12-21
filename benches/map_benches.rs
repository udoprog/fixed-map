use criterion::{Bencher, Benchmark, Criterion};
use std::mem;

/// Macro to build a benchmark.
macro_rules! benches {
    (
    $({
        $len:expr, ($($member:ident),*), ($($insert:ident),*), $get:ident
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

                // Assert that size of Key is identical to array.
                assert_eq!(
                    mem::size_of::<<Key as fixed_map::Key<Key, usize>>::Storage>(),
                    mem::size_of::<[Option<usize>; $len]>(),
                );

                let mut it = 1u32..;
                let mut map = fixed_map::Map::new();
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
                #[repr(usize)]
                pub enum Key {
                    $($member,)*
                }

                let mut it = 1u32..;
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

                let mut it = 1u32..;
                let mut map = hashbrown::HashMap::with_capacity($len);
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

                b.iter(|| {
                    let mut map = fixed_map::Map::<Key, u32>::new();
                    $(map.insert(Key::$insert, 42u32);)*
                    ($(map.get(Key::$insert).cloned(),)*)
                })
            }),
        );
    )*

    $(
        criterion.bench(
            "array",
            Benchmark::new(concat!("insert", stringify!($len)), |b: &mut Bencher| {
                #[allow(unused)]
                #[repr(usize)]
                pub enum Key {
                    $($member,)*
                }

                b.iter(|| {
                    let mut map = [None; $len];
                    $(map[Key::$insert as usize] = Some(42u32);)*
                    ($(map[Key::$insert as usize].as_ref().cloned(),)*)
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

                b.iter(|| {
                    let mut map = hashbrown::HashMap::<_, u32>::with_capacity($len);
                    $(map.insert(Key::$insert, 42u32);)*
                    ($(map.get(&Key::$insert).cloned(),)*)
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
                #[repr(usize)]
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
        (T00, T01, T02, T03),
        (T00, T03),
        T03
    };

    {
        8,
        (T00, T01, T02, T03, T04, T05, T06, T07),
        (T00, T03, T06),
        T03
    };

    {
        16,
        (T00, T01, T02, T03, T04, T05, T06, T07, T8, T9, T10, T11, T12, T13, T14, T15),
        (T00, T03, T06, T12, T14),
        T14
    };

    {
        32,
        (
            T00, T01, T02, T03, T04, T05, T06, T07, T08, T09, T10, T11, T12, T13, T14, T15,
            T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
        ),
        (T00, T03, T06, T12, T14, T23, T28, T31),
        T28
    };
}

criterion::criterion_group! {
    name = map_group;
    config = Criterion::default();
    targets = benches
}

criterion::criterion_main!(map_group);
