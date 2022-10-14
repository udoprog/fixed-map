// This file is very useful for debugging the derive macro.
// Doctests don't work well for that purpose.

fn main() {
    #[cfg(feature = "map")]
    {
        use fixed_map::{Key, Map};

        #[derive(Clone, Copy, Key)]
        enum Part {
            One,
            Two,
        }

        #[derive(Clone, Copy, Key)]
        enum Key {
            Simple,
            Composite(Part),
            String(&'static str),
            Number(u32),
            Singleton(()),
        }
        let mut map = Map::new();

        map.insert(Key::Simple, 1);
        map.insert(Key::Composite(Part::One), 2);
        map.insert(Key::String("foo"), 3);
        map.insert(Key::Number(1), 4);
        map.insert(Key::Singleton(()), 5);

        assert_eq!(map.get(Key::Simple), Some(&1));
        assert_eq!(map.get(Key::Composite(Part::One)), Some(&2));
        assert_eq!(map.get(Key::Composite(Part::Two)), None);
        assert_eq!(map.get(Key::String("foo")), Some(&3));
        assert_eq!(map.get(Key::String("bar")), None);
        assert_eq!(map.get(Key::Number(1)), Some(&4));
        assert_eq!(map.get(Key::Number(2)), None);
        assert_eq!(map.get(Key::Singleton(())), Some(&5));
    }
}

// Execute this during testing as well.
#[cfg(test)]
#[test]
fn test_main() {
    main();
}
