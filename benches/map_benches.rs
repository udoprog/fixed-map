use criterion::{Bencher, BenchmarkId, Criterion};
use std::mem;

/// Macro to build a benchmark.
macro_rules! benches {
    (
    $({
        $len:expr, ($($member:ident),*), ($($insert:ident),*), $get:ident
    };)*
    ) => {
    fn benches(criterion: &mut Criterion) {
        {
            let mut group = criterion.benchmark_group("get");

            $(
                group.bench_with_input(BenchmarkId::new("fixed", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[derive(Clone, Copy, fixed_map::Key)]
                    pub enum Key { $($member,)* }

                    // Assert that size of Key is identical to array.
                    assert_eq!(
                        mem::size_of::<<Key as fixed_map::key::Key<Key, usize>>::Storage>(),
                        mem::size_of::<[Option<usize>; $len]>(),
                    );

                    let mut it = 1u32..;
                    let mut map = fixed_map::Map::new();
                    $(map.insert(Key::$insert, it.next().unwrap());)*

                    b.iter(|| map.get(Key::$get))
                });
            )*

            $(
                group.bench_with_input(BenchmarkId::new("hashbrown", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[derive(PartialEq, Eq, Hash)]
                    pub enum Key { $($member,)* }

                    let mut it = 1u32..;
                    let mut map = hashbrown::HashMap::with_capacity($len);
                    $(map.insert(Key::$insert, it.next().unwrap());)*

                    b.iter(|| map.get(&Key::$get))
                });
            )*

            $(
                group.bench_with_input(BenchmarkId::new("array", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[repr(usize)]
                    pub enum Key { $($member,)* }

                    let mut it = 1u32..;
                    let mut map = [None; $len];
                    $(map[Key::$insert as usize] = Some(it.next().unwrap());)*

                    b.iter(|| map[Key::$get as usize])
                });
            )*
        }

        {
            let mut group = criterion.benchmark_group("insert");

            $(
                group.bench_with_input(BenchmarkId::new("fixed", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[derive(Clone, Copy, fixed_map::Key)]
                    pub enum Key { $($member,)* }

                    b.iter(|| {
                        let mut map = fixed_map::Map::<Key, u32>::new();
                        $(map.insert(Key::$insert, 42u32);)*
                        ($(map.get(Key::$insert).cloned(),)*)
                    })
                });
            )*

            $(
                group.bench_with_input(BenchmarkId::new("hashbrown", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[derive(Clone, PartialEq, Eq, Hash)]
                    pub enum Key { $($member,)* }

                    b.iter(|| {
                        let mut map = hashbrown::HashMap::<_, u32>::with_capacity($len);
                        $(map.insert(Key::$insert, 42u32);)*
                        ($(map.get(&Key::$insert).cloned(),)*)
                    })
                });
            )*

            $(
                group.bench_with_input(BenchmarkId::new("array", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[repr(usize)]
                    pub enum Key { $($member,)* }

                    b.iter(|| {
                        let mut map = [None; $len];
                        $(map[Key::$insert as usize] = Some(42u32);)*
                        ($(map[Key::$insert as usize].as_ref().cloned(),)*)
                    })
                });
            )*
        }

        {
            let mut group = criterion.benchmark_group("iter");

            $(
                group.bench_with_input(BenchmarkId::new("fixed", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[derive(Clone, Copy, fixed_map::Key)]
                    pub enum Key { $($member,)* }

                    let mut it = 1u32..;
                    let mut map = fixed_map::Map::new();
                    $(map.insert(Key::$insert, it.next().unwrap());)*

                    b.iter(|| map.values().cloned().sum::<u32>())
                });
            )*

            $(
                group.bench_with_input(BenchmarkId::new("hashbrown", $len), &$len, |b: &mut Bencher, len| {
                    #[allow(unused)]
                    #[derive(PartialEq, Eq, Hash)]
                    pub enum Key { $($member,)* }

                    let mut it = 1u32..;
                    let mut map = hashbrown::HashMap::with_capacity(*len);
                    $(map.insert(Key::$insert, it.next().unwrap());)*

                    b.iter(|| map.values().cloned().sum::<u32>())
                });
            )*

            $(
                group.bench_with_input(BenchmarkId::new("array", $len), &$len, |b: &mut Bencher, _| {
                    #[allow(unused)]
                    #[repr(usize)]
                    pub enum Key { $($member,)* }

                    let mut it = 1u32..;
                    let mut map = [None; $len];
                    $(map[Key::$insert as usize] = Some(it.next().unwrap());)*
                    b.iter(|| map.iter().flat_map(|v| v.clone()).sum::<u32>())
                });
            )*
        }
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
