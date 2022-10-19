use crate::storage::Storage;

mod boolean;
#[cfg(feature = "map")]
mod map;
mod option;
mod singleton;

pub trait OccupiedEntry<'this, K, V> {
    fn key(&self) -> K;
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
    fn into_mut(self) -> &'this mut V;
    fn insert(&mut self, value: V) -> V;
    fn remove(self) -> V;
}

pub trait VacantEntry<'this, K, V> {
    fn key(&self) -> K;
    fn insert(self, value: V) -> &'this mut V;
}

pub enum Entry<Occupied, Vacant> {
    Occupied(Occupied),
    Vacant(Vacant),
}

impl<Occupied, Vacant> Entry<Occupied, Vacant> {
    pub fn or_insert<'this, K, V>(self, default: V) -> &'this mut V
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<'this, K, V, F: FnOnce() -> V>(self, default: F) -> &'this mut V
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn or_insert_with_key<'this, K, V, F: FnOnce(K) -> V>(self, default: F) -> &'this mut V
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    pub fn key<'this, K, V>(&self) -> K
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    #[allow(clippy::return_self_not_must_use)]
    pub fn and_modify<'this, K, V, F: FnOnce(&mut V)>(self, f: F) -> Self
    where
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }

    pub fn or_default<'this, K, V>(self) -> &'this mut V
    where
        V: Default,
        Occupied: OccupiedEntry<'this, K, V>,
        Vacant: VacantEntry<'this, K, V>,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

pub trait StorageEntry<'this, K, V>: Storage<K, V>
where
    Self: 'this,
    Self::Occupied: OccupiedEntry<'this, K, V>,
    Self::Vacant: VacantEntry<'this, K, V>,
{
    type Occupied;
    type Vacant;

    fn entry(&'this mut self, key: K) -> Entry<Self::Occupied, Self::Vacant>;
}
