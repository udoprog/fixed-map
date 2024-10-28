use core::mem;

use criterion::{Bencher, BenchmarkId, Criterion};

macro_rules! benches {
    (
    $({
        $len:expr, ($($member:ident),* $(,)?),
        insert = [$($insert:ident),* $(,)?],
        get => $get:ident,
        entry => [$($entry:ident),* $(,)?],
    };)*
    ) => {
    fn get_benches(criterion: &mut Criterion) {
        let mut group = criterion.benchmark_group("get");

        $({
            #[allow(unused)]
            #[derive(Clone, Copy, fixed_map::Key)]
            pub enum Key { $($member,)* }

            // Assert that size of Key is identical to array.
            const _: () = assert!(
                mem::size_of::<<Key as fixed_map::Key>::MapStorage<usize>>() == mem::size_of::<[Option<usize>; $len]>()
            );

            group.bench_with_input(BenchmarkId::new("fixed", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = fixed_map::Map::new();
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| map.get(Key::$get))
            });
        })*

        $({
            #[allow(unused)]
            #[derive(PartialEq, Eq, Hash)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("hashbrown", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = hashbrown::HashMap::with_capacity($len);
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| map.get(&Key::$get))
            });
        })*

        $({
            #[allow(unused)]
            #[repr(usize)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("array", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = [None; $len];
                $(map[Key::$insert as usize] = Some(it.next().unwrap());)*

                b.iter(|| map[Key::$get as usize])
            });
        })*
    }

    fn insert_benches(criterion: &mut Criterion) {
        let mut group = criterion.benchmark_group("insert");

        $({
            #[allow(unused)]
            #[derive(Clone, Copy, fixed_map::Key)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("fixed", $len), &$len, |b: &mut Bencher, _| {
                b.iter(|| {
                    let mut map = fixed_map::Map::<Key, u32>::new();
                    $(map.insert(Key::$insert, 42u32);)*
                    ($(map.get(Key::$insert).cloned(),)*)
                })
            });
        })*

        $({
            #[allow(unused)]
            #[derive(Clone, PartialEq, Eq, Hash)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("hashbrown", $len), &$len, |b: &mut Bencher, _| {
                b.iter(|| {
                    let mut map = hashbrown::HashMap::<_, u32>::with_capacity($len);
                    $(map.insert(Key::$insert, 42u32);)*
                    ($(map.get(&Key::$insert).cloned(),)*)
                })
            });
        })*

        $({
            #[allow(unused)]
            #[repr(usize)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("array", $len), &$len, |b: &mut Bencher, _| {
                b.iter(|| {
                    let mut map = [None; $len];
                    $(map[Key::$insert as usize] = Some(42u32);)*
                    ($(map[Key::$insert as usize].as_ref().cloned(),)*)
                })
            });
        })*
    }

    fn values_benches(criterion: &mut Criterion) {
        let mut group = criterion.benchmark_group("values");

        $({
            #[allow(unused)]
            #[derive(Clone, Copy, fixed_map::Key)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("fixed", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = fixed_map::Map::new();
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| map.values().copied().sum::<u32>())
            });
        })*

        $({
            #[allow(unused)]
            #[derive(PartialEq, Eq, Hash)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("hashbrown", $len), &$len, |b: &mut Bencher, len| {
                let mut it = 1u32..;
                let mut map = hashbrown::HashMap::with_capacity(*len);
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| map.values().copied().sum::<u32>())
            });
        })*

        $({
            #[allow(unused)]
            #[repr(usize)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("array", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = [None; $len];
                $(map[Key::$insert as usize] = Some(it.next().unwrap());)*

                b.iter(|| map.iter().flatten().copied().sum::<u32>())
            });
        })*
    }

    fn entry_benches(criterion: &mut Criterion) {
        let mut group = criterion.benchmark_group("entry");

        $({
            #[allow(unused)]
            #[derive(Clone, Copy, fixed_map::Key)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("fixed", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = fixed_map::Map::new();
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| {
                    let mut value = 0;
                    let mut map = map.clone();

                    $({
                        let v = map.entry(Key::$entry).or_default();
                        *v += 1;
                        value += *v;
                    })*

                    value
                });
            });
        })*

        $({
            #[allow(unused)]
            #[derive(Clone, PartialEq, Eq, Hash)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("hashbrown", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = hashbrown::HashMap::with_capacity($len);
                $(map.insert(Key::$insert, it.next().unwrap());)*

                b.iter(|| {
                    let mut map = map.clone();
                    let mut value = 0;

                    $({
                        let v = map.entry(Key::$entry).or_default();
                        *v += 1;
                        value += *v;
                    })*

                    value
                });
            });
        })*

        $({
            #[allow(unused)]
            #[repr(usize)]
            pub enum Key { $($member,)* }

            group.bench_with_input(BenchmarkId::new("array", $len), &$len, |b: &mut Bencher, _| {
                let mut it = 1u32..;
                let mut map = [None; $len];
                $(map[Key::$insert as usize] = Some(it.next().unwrap());)*

                b.iter(|| {
                    let mut map = map;
                    let mut value = 0;

                    $({
                        let v = map[Key::$entry as usize].get_or_insert_with(Default::default);
                        *v += 1;
                        value += *v;
                    })*

                    value
                });
            });
        })*
    }}
}

benches! {
    {
        4,
        (T00, T01, T02, T03),
        insert = [T00, T03],
        get => T03,
        entry => [T02, T03],
    };

    {
        8,
        (T00, T01, T02, T03, T04, T05, T06, T07),
        insert = [T00, T03, T06],
        get => T03,
        entry => [T02, T03],
    };

    {
        16,
        (T00, T01, T02, T03, T04, T05, T06, T07, T8, T9, T10, T11, T12, T13, T14, T15),
        insert = [T00, T03, T06, T12, T14],
        get => T14,
        entry => [T02, T14],
    };

    {
        32,
        (
            T00, T01, T02, T03, T04, T05, T06, T07, T08, T09, T10, T11, T12, T13, T14, T15,
            T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
        ),
        insert = [T00, T03, T06, T12, T14, T23, T28, T31],
        get => T28,
        entry => [T11, T28],
    };
}

criterion::criterion_group! {
    name = complex;
    config = Criterion::default();
    targets = get_benches, insert_benches, values_benches, entry_benches
}

criterion::criterion_main!(complex);
