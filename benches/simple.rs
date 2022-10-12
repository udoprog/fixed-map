use criterion::Criterion;

macro_rules! expand {
    ($len:expr, ($($member:ident),*), $get:ident) => {
        #[allow(unused)]
        #[derive(Clone, Copy, fixed_map::Key)]
        pub enum FixedKey {
            $($member,)*
        }

        #[no_mangle]
        #[inline(never)]
        pub fn sum_fixed(map: &fixed_map::Map<FixedKey, u32>) -> u32 {
            map.values().copied().sum()
        }

        #[allow(unused)]
        #[repr(usize)]
        pub enum ArrayKey {
            $($member,)*
        }

        #[no_mangle]
        #[inline(never)]
        pub fn test_fixed(map: &[Option<u32>; $len]) -> u32 {
            map.iter().flat_map(|v| v).copied().sum()
        }
    }
}

expand! {
    16,
    (T00, T01, T02, T03, T04, T05, T06, T07, T8, T9, T10, T11, T12, T13, T14, T15),
    T14
}

fn benches(criterion: &mut Criterion) {
    {
        let mut group = criterion.benchmark_group("array");

        group.bench_function("sum_values", |iter| {
            let mut array = [None; 16];
            array[ArrayKey::T07 as usize] = Some(4);
            array[ArrayKey::T10 as usize] = Some(13);

            iter.iter(|| test_fixed(&array))
        });
    }

    {
        let mut group = criterion.benchmark_group("fixed");

        group.bench_function("sum_values", |iter| {
            let mut map = fixed_map::Map::<_, u32>::new();
            map.insert(FixedKey::T07, 4);
            map.insert(FixedKey::T10, 13);

            iter.iter(|| sum_fixed(&map))
        });
    }
}

criterion::criterion_group! {
    name = simple;
    config = Criterion::default();
    targets = benches
}

criterion::criterion_main!(simple);
