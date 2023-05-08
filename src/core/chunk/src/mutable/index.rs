use std::hash::Hash;

use croaring::Bitmap;
use hashbrown::HashMap;
use pdatastructs::filters::{bloomfilter::BloomFilter, Filter};

// #[derive(PartialEq, Debug)]
// pub enum Set {
//     Universe,
//     Some(Bitmap),
// }

// impl Set {
//     pub fn clear(&mut self) {
//         match self {
//             Set::Universe => *self = Self::Some(Bitmap::create()),
//             Set::Some(set) => set.clear(),
//         }
//     }

//     pub fn and_inplace(&mut self, rhs: Self) {
//         match self {
//             Set::Universe => match rhs {
//                 Set::Universe => {}
//                 Set::Some(_) => *self = rhs,
//             },
//             Set::Some(lhs) => match rhs {
//                 Set::Universe => {}
//                 Set::Some(rhs) => lhs.and_inplace(&rhs),
//             },
//         }
//     }

//     pub fn andnot_inplace(&mut self, rhs: Self) {
//         match self {
//             Set::Universe => match rhs {
//                 Set::Universe => *self = Set::Some(Bitmap::create()),
//                 Set::Some(_) => {
//                     (0..self.)
//                 }
//             },
//             Set::Some(lhs) => match rhs {
//                 Set::Universe => {}
//                 Set::Some(rhs) => lhs.andnot_inplace(&rhs),
//             },
//         }
//     }
// }

pub trait Index {
    type Value;

    fn lookup<F: FnMut(&Bitmap)>(&self, value: &Self::Value, f: F);
    fn insert(&mut self, row: u32, value: Self::Value);
    fn exactly(&self) -> bool;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InvertedIndex<V>
where
    V: Eq + Hash,
{
    data: HashMap<V, Bitmap>,
}

impl<V> InvertedIndex<V>
where
    V: Eq + Hash,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<V> Index for InvertedIndex<V>
where
    V: Eq + Hash,
{
    type Value = V;

    #[inline]
    fn lookup<F: FnMut(&Bitmap)>(&self, value: &Self::Value, mut f: F) {
        self.data.get(value).map(|set| (f)(set));
    }

    #[inline]
    fn insert(&mut self, row: u32, value: Self::Value) {
        let bitmap = self.data.entry(value).or_insert_with(|| Bitmap::create());
        bitmap.add(row);
    }

    #[inline]
    fn exactly(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct SparseIndex<V: Hash> {
    seens: Vec<BloomFilter<V>>,
    block_size: u32,
}

impl<T: Hash> SparseIndex<T> {
    #[inline]
    pub(crate) fn new(block_size: u32) -> Self {
        Self {
            seens: Vec::new(),
            block_size,
        }
    }
}

impl<V: Hash> Index for SparseIndex<V> {
    type Value = V;

    #[inline]
    fn lookup<F: FnMut(&Bitmap)>(&self, value: &Self::Value, mut f: F) {
        let mut bitmap = Bitmap::create();
        for (offset, block) in self.seens.iter().enumerate() {
            if block.query(value) {
                let offset = offset as u32;
                bitmap.add_range((offset * self.block_size)..((offset + 1) * self.block_size));
            }
        }

        (f)(&bitmap);
    }

    #[inline]
    fn insert(&mut self, row: u32, value: Self::Value) {
        let block = (row / self.block_size) as usize;
        if self.seens.len() <= block {
            self.seens.resize_with(block + 1, || {
                BloomFilter::with_properties(self.block_size as usize, 1.0 / 100.0)
            });
        }
        self.seens[block].insert(&value).unwrap();
    }

    #[inline]
    fn exactly(&self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum IndexType<Inverted, Sparse> {
    Inverted(Inverted),
    Sparse(Sparse),
}

pub type IndexImpl<V> = IndexType<InvertedIndex<V>, SparseIndex<V>>;

impl<V> IndexImpl<V>
where
    V: Eq + Hash,
{
    pub fn new(data_type: IndexType<(), u32>) -> Self {
        match data_type {
            IndexType::Inverted(_) => IndexImpl::Inverted(InvertedIndex::new()),
            IndexType::Sparse(block_size) => IndexImpl::Sparse(SparseIndex::new(block_size)),
        }
    }

    pub fn lookup<F: FnMut(&Bitmap)>(&self, value: &V, f: F) {
        match self {
            IndexType::Inverted(index) => index.lookup(value, f),
            IndexType::Sparse(index) => index.lookup(value, f),
        }
    }

    pub fn exactly(&self) -> bool {
        match self {
            IndexType::Inverted(index) => index.exactly(),
            IndexType::Sparse(index) => index.exactly(),
        }
    }

    pub fn insert(&mut self, id: usize, v: V) {
        match self {
            IndexType::Inverted(index) => index.insert(id as u32, v),
            IndexType::Sparse(index) => index.insert(id as u32, v),
        }
    }
}

#[cfg(test)]
mod tests {
    use croaring::Bitmap;
    use pdatastructs::filters::bloomfilter::BloomFilter;

    use super::{Index, SparseIndex};
    use crate::mutable::index::InvertedIndex;

    #[test]
    fn test_bloom_filter() {
        // - input size: we expect 10M elements
        // - reliability: probability of false positives should be <= 1%
        // - CPU efficiency: number of hash functions should be <= 10
        // - RAM efficiency: size should be <= 15MB
        let seen = BloomFilter::<u64>::with_properties(10_000_000, 1.0 / 100.0);
        const BOUND_HASH_FUNCTIONS: usize = 10;
        assert!(
            seen.k() <= BOUND_HASH_FUNCTIONS,
            "number of hash functions for bloom filter should be <= {} but is {}",
            BOUND_HASH_FUNCTIONS,
            seen.k(),
        );
        const BOUND_SIZE_BYTES: usize = 15_000_000;
        let size_bytes = (seen.m() + 7) / 8;
        assert!(
            size_bytes <= BOUND_SIZE_BYTES,
            "size of bloom filter should be <= {} bytes but is {} bytes",
            BOUND_SIZE_BYTES,
            size_bytes,
        );
    }

    #[test]
    fn test_sparse_index() {
        let mut index = SparseIndex::<usize>::new(1000);
        index.insert(0, 1);
        index.insert(1001, 1);
        index.insert(2001, 2);
        let mut result = Bitmap::from_range(0..=2001);
        index.lookup(&1, |set| result.and_inplace(set));
        let mut expect = Bitmap::create();
        expect.add_range(0..2000);
        assert_eq!(result, expect);
    }

    #[test]
    fn test_inverted_index() {
        let mut index = InvertedIndex::<usize>::new();
        index.insert(0, 1);
        index.insert(1001, 1);
        let mut result = Bitmap::from_range(0..=1001);
        index.lookup(&1, |set| result.and_inplace(set));
        let mut expect = Bitmap::create();
        expect.add(0);
        expect.add(1001);
        assert_eq!(result, expect);
    }

    #[test]
    fn test_fusion_index() {
        let mut index_1 = SparseIndex::<usize>::new(1);
        let mut index_2 = InvertedIndex::<usize>::new();
        index_1.insert(0, 0);
        index_1.insert(1, 1);
        index_2.insert(0, 1);
        index_2.insert(1, 1);
        let mut result = Bitmap::from_range(0..=4);
        index_1.lookup(&1, |set| result.and_inplace(set));
        index_2.lookup(&1, |set| result.and_inplace(set));
        let mut b = Bitmap::create();
        b.add(1);
        assert_eq!(result, b);
    }
}
