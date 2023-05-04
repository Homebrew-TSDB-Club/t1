use std::ops::Range;

use super::{Scalar, ScalarMut, ScalarRef};
use crate::{
    array::bitmap::{Bitmap, BitmapRef, BitmapRefMut},
    primitive::Primitive,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct NfsList<P: Primitive> {
    pub(crate) data: Vec<P>,
    pub(crate) validity: Bitmap,
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

    #[inline]
    fn as_ref(&self) -> Self::Ref<'_> {
        NfsSlice::new(self.validity.as_ref(), &self.data[..])
    }
}

impl<P: Primitive> From<Vec<Option<P>>> for NfsList<P> {
    fn from(raw: Vec<Option<P>>) -> Self {
        let mut validity = Bitmap::new();
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
        validity.align();
        Self { data, validity }
    }
}

#[derive(Debug, PartialEq)]
pub struct NfsSlice<'a, P: Primitive> {
    pub(crate) validity: BitmapRef<'a>,
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
    #[inline]
    pub(crate) fn new(validity: BitmapRef<'a>, data: &'a [P]) -> Self {
        Self { validity, data }
    }

    #[inline]
    pub fn get(&self, n: usize) -> Option<Option<&P>> {
        if self.data.len() > n {
            if self.validity.get_bit(n) {
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
            validity: self.validity.slice(range.start, range.end),
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
    pub(crate) validity: BitmapRefMut<'a>,
    pub(crate) data: &'a mut [P],
}

impl<'a, P: Primitive> ScalarMut<'a> for NfsSliceMut<'a, P> {
    type Owned = NfsList<P>;
}

impl<'a, P: Primitive> NfsSliceMut<'a, P> {
    #[inline]
    pub(crate) fn new(validity: BitmapRefMut<'a>, data: &'a mut [P]) -> Self {
        Self { validity, data }
    }

    #[inline]
    pub fn get(&self, n: usize) -> Option<Option<&P>> {
        if self.validity.len() > n {
            if self.validity.get_bit(n) {
                Some(Some(&self.data[n]))
            } else {
                Some(None)
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn insert(&mut self, n: usize, value: Option<P>) {
        self.validity.insert(n, value.is_some());
        if let Some(value) = value {
            self.data[n] = value;
        }
    }
}
