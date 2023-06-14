use std::hash::{BuildHasher, Hash, Hasher};

use ahash::RandomState;
use hashbrown::{hash_map::RawEntryMut, HashMap};

use super::Array;
use crate::scalar::Scalar;

#[inline]
fn hash_with_state<H: Hash>(state: &RandomState, value: &H) -> u64 {
    let mut hasher = state.build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone)]
pub struct Dictionary<A> {
    hash_state: RandomState,
    dedup: HashMap<usize, (), ()>,
    data: A,
}

impl<A> Dictionary<A> {
    #[inline]
    pub(crate) fn new(data: A) -> Self {
        Self {
            hash_state: RandomState::new(),
            dedup: Default::default(),
            data,
        }
    }
}

impl<A: Array> Dictionary<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    #[inline]
    pub(crate) fn lookup_or_insert(&mut self, value: A::Item) -> usize {
        let hash = hash_with_state(&self.hash_state, &value.as_ref());
        let entry = self.dedup.raw_entry_mut().from_hash(hash, |key| {
            value.as_ref() == unsafe { self.data.get_unchecked(*key) }
        });

        return match entry {
            RawEntryMut::Occupied(entry) => *entry.into_key(),
            RawEntryMut::Vacant(entry) => {
                self.data.push(value);
                *entry
                    .insert_with_hasher(hash, self.data.len() - 1, (), |index| {
                        let list = self.data.get(*index).unwrap();
                        hash_with_state(&self.hash_state, &list)
                    })
                    .0
            }
        } + 1;
    }

    #[allow(unused)]
    #[inline]
    pub(crate) fn lookup(&self, value: A::ItemRef<'_>) -> Option<usize> {
        return self
            .dedup
            .raw_entry()
            .from_hash(hash_with_state(&self.hash_state, &value), |key| unsafe {
                self.data.get_unchecked(*key) == value
            })
            .map(|(&symbol, &())| symbol + 1);
    }

    #[inline]
    pub(crate) fn get(&self, id: usize) -> Option<Option<A::ItemRef<'_>>> {
        if id == 0 {
            Some(None)
        } else {
            Some(self.data.get(id - 1))
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, id: usize) -> Option<Option<A::ItemMut<'_>>> {
        if id == 0 {
            Some(None)
        } else {
            Some(self.data.get_mut(id - 1))
        }
    }

    #[inline]
    pub(crate) unsafe fn get_unchecked(&self, id: usize) -> Option<A::ItemRef<'_>> {
        if id == 0 {
            None
        } else {
            Some(self.data.get_unchecked(id - 1))
        }
    }

    #[inline]
    pub(crate) unsafe fn get_unchecked_mut(&mut self, id: usize) -> Option<A::ItemMut<'_>> {
        if id == 0 {
            None
        } else {
            Some(self.data.get_unchecked_mut(id - 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Dictionary;
    use crate::array::list::ListArray;

    #[test]
    fn test_storage() {
        let mut dict = Dictionary::<ListArray<u8>>::new(ListArray::new());
        let id = dict.lookup_or_insert(Vec::from("hello, world"));
        let id2 = dict.lookup_or_insert(Vec::from("hello, world"));
        assert_eq!(id, id2);
        let id3 = dict.lookup_or_insert(Vec::from("hello world"));
        assert_ne!(id, id3);
        let v1 = dict.get(id);
        let v2 = dict.get(id2);
        assert_eq!(v1, v2);
        let id = dict.lookup("hello, world".as_ref());
        assert_eq!(id, Some(1));
    }
}
