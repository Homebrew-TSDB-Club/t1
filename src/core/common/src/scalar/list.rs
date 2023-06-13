use std::ops::Range;

use bitvec::prelude::*;

use super::{Scalar, ScalarMut, ScalarRef};
use crate::primitive::Primitive;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct NfsList<P: Primitive> {
    pub(crate) data: Vec<P>,
    pub(crate) validity: BitVec,
}

impl<P: Primitive> NfsList<P> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<P: Primitive> Scalar for NfsList<P> {
    type Ref<'a> = NfsSlice<'a, P>;
    type RefMut<'a> = NfsSliceMut<'a, P>;

    fn as_ref(&self) -> Self::Ref<'_> {
        NfsSlice::new(self.validity.as_ref(), &self.data[..])
    }
}

impl<P: Primitive> From<Vec<Option<P>>> for NfsList<P> {
    fn from(raw: Vec<Option<P>>) -> Self {
        let mut validity = BitVec::new();
        let mut data = Vec::new();
        for item in raw {
            if let Some(item) = item {
                validity.push(true);
                data.push(item);
            } else {
                validity.push(false);
                data.push(Default::default());
            }
        }
        Self { data, validity }
    }
}

#[derive(Debug, PartialEq)]
pub struct NfsSlice<'a, P: Primitive> {
    pub(crate) validity: &'a BitSlice,
    pub(crate) data: &'a [P],
}

impl<'a, P: Primitive> ScalarRef<'a> for NfsSlice<'a, P> {
    type Owned = NfsList<P>;

    #[inline]
    fn to_owned(self) -> Self::Owned {
        NfsList {
            data: ToOwned::to_owned(self.data),
            validity: self.validity.to_owned(),
        }
    }
}

impl<P: Primitive> Clone for NfsSlice<'_, P> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            validity: self.validity.clone(),
            data: self.data,
        }
    }
}

impl<'a, P: Primitive> NfsSlice<'a, P> {
    pub(crate) fn new(validity: &'a BitSlice, data: &'a [P]) -> Self {
        Self { validity, data }
    }

    #[inline]
    pub fn get(&self, n: usize) -> Option<Option<&P>> {
        if let Some(bit) = self.validity.get(n) {
            if *bit {
                Some(Some(&self.data[n]))
            } else {
                Some(None)
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn slice(&'a self, range: Range<usize>) -> NfsSlice<'a, P> {
        Self {
            validity: &self.validity[range.clone()],
            data: &self.data[range],
        }
    }
}

impl<'a, P: Primitive> IntoIterator for NfsSlice<'a, P> {
    type Item = Option<P>;

    type IntoIter = NfsSliceIter<'a, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        NfsSliceIter { slice: self, n: 0 }
    }
}

pub struct NfsSliceIter<'a, P: Primitive> {
    slice: NfsSlice<'a, P>,
    n: usize,
}

impl<'a, P: Primitive> Iterator for NfsSliceIter<'a, P> {
    type Item = Option<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.n;
        self.n += 1;
        self.slice.get(n).map(|v| v.copied())
    }
}

#[derive(Debug, PartialEq)]
pub struct NfsSliceMut<'a, P: Primitive> {
    pub(crate) validity: &'a mut BitSlice,
    pub(crate) data: &'a mut [P],
}

impl<'a, P: Primitive> ScalarMut<'a> for NfsSliceMut<'a, P> {
    type Owned = NfsList<P>;
}

impl<'a, P: Primitive> NfsSliceMut<'a, P> {
    pub(crate) fn new(validity: &'a mut BitSlice, data: &'a mut [P]) -> Self {
        Self { validity, data }
    }

    #[inline]
    pub fn get(&self, n: usize) -> Option<Option<&P>> {
        if let Some(bit) = self.validity.get(n) {
            if *bit {
                Some(Some(&self.data[n]))
            } else {
                Some(None)
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, n: usize, value: Option<P>) {
        self.validity.set(n, value.is_some());
        if let Some(value) = value {
            self.data[n] = value;
        }
    }
}
