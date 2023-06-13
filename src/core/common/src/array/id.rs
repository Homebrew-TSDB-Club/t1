use std::hash::Hash;

use super::{dictionary::Dictionary, Array};

#[derive(Debug, Clone)]
pub struct IdArray<A> {
    values: Dictionary<A>,
    data: Vec<usize>,
}

impl<A: Array> IdArray<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    pub fn lookup_id(&self, value: A::ItemRef<'_>) -> Option<usize> {
        self.values.lookup(value)
    }

    pub fn push_and_get_id(&mut self, value: <Self as Array>::Item) -> usize {
        match value {
            Some(value) => {
                let valud_id = self.values.lookup_or_insert(value);
                self.data.push(valud_id);
                valud_id
            }
            None => {
                self.push_zero();
                0
            }
        }
    }
}

impl<A: Array> IdArray<A> {
    #[inline]
    pub fn new(array: A) -> Self {
        Self {
            values: Dictionary::new(array),
            data: Vec::<usize>::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize, array: A) -> Self {
        Self {
            values: Dictionary::new(array),
            data: Vec::<usize>::with_capacity(capacity),
        }
    }
}

impl<A: Array> Array for IdArray<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    type Item = Option<A::Item>;
    type ItemRef<'a> = Option<A::ItemRef<'a>>;
    type ItemRefMut<'a> = Option<A::ItemRefMut<'a>>;

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        let vid = self.data.get(id)?;
        self.values.get(*vid)
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        self.values.get_unchecked(self.data[id])
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        let vid = self.data.get(id)?;
        self.values.get_mut(*vid)
    }

    #[inline]
    fn push(&mut self, value: Self::Item) {
        match value {
            Some(value) => {
                self.data.push(self.values.lookup_or_insert(value));
            }
            None => {
                self.push_zero();
            }
        }
    }

    #[inline]
    fn push_zero(&mut self) {
        self.data.push(0);
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}
