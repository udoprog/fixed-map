use option_bucket::*;

struct OccupiedEntry<'a, T> {
    inner: SomeBucket<'a, T>,
}
struct VacantEntry<'a, T> {
    inner: NoneBucket<'a, T>,
}

enum Entry<'a, T> {
    Vacant(VacantEntry<'a, T>),
    Occupied(OccupiedEntry<'a, T>),
}

impl<'a, T> VacantEntry<'a, T> {
    fn insert(self, value: T) -> &'a mut T {
        self.inner.insert(value)
    }
}

impl<'a, T> OccupiedEntry<'a, T> {
    fn get(&self) -> &T {
        self.inner.as_ref()
    }

    fn get_mut(&mut self) -> &mut T {
        self.inner.as_mut()
    }

    fn into_mut(self) -> &'a mut T {
        self.inner.into_mut()
    }

    fn insert(&mut self, value: T) -> T {
        self.inner.replace(value)
    }

    fn remove(self) -> T {
        self.inner.take()
    }
}

impl<'a, T> Entry<'a, T> {
    pub fn or_insert(self, default: T) -> &'a mut T {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> T>(self, default: F) -> &'a mut T {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn and_modify<F: FnOnce(&mut T)>(self, f: F) -> Self {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }

    pub fn or_default(self) -> &'a mut T
    where
        T: Default,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

trait OptionEntry {
    type Entry<'this>
    where
        Self: 'this;
    fn entry(&mut self) -> Self::Entry<'_>;
}

impl<T> OptionEntry for Option<T> {
    type Entry<'this> = Entry<'this, T> where Self: 'this;
    fn entry(&mut self) -> Self::Entry<'_> {
        match OptionBucket::new(self) {
            OptionBucket::Some(inner) => Entry::Occupied(OccupiedEntry { inner }),
            OptionBucket::None(inner) => Entry::Vacant(VacantEntry { inner }),
        }
    }
}

#[test]
fn test() {
    let mut even: Option<i32> = None;

    for n in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
        if n % 2 == 0 {
            even.entry().and_modify(|x| *x += 1).or_insert(1);
        }
    }

    match even.entry() {
        Entry::Occupied(mut entry) => {
            assert_eq!(entry.get(), &5);
            assert_eq!(entry.insert(-3), 5);
            assert_eq!(entry.remove(), -3);
        }
        Entry::Vacant(_) => unreachable!(),
    }
    assert!(even.is_none());

    let mut x = None;
    x.entry().or_insert_with(|| vec![String::new()]);
    assert_eq!(x, Some(vec![String::new()]));

    let mut y: Option<u32> = None;
    y.entry().or_default();
    assert_eq!(y, Some(0));
}
