use std::ops::{Range, RangeFrom};

use bitvec::prelude::*;

use super::{Scalar, ScalarMut, ScalarRef};
use crate::primitive::Primitive;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct OptionalFixedList<P: Primitive> {
    pub(crate) data: Vec<P>,
    pub(crate) validity: BitVec,
}

impl<P: Primitive> OptionalFixedList<P> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn pad(mut self, item: P) -> Vec<P> {
        for (offset, some) in self.validity.into_iter().enumerate() {
            if !some {
                self.data[offset] = item;
            }
        }
        self.data
    }

    #[inline]
    pub fn push(&mut self, item: Option<P>) {
        match item {
            Some(item) => {
                self.data.push(item);
                self.validity.push(true);
            }
            None => {
                self.data.push(P::default());
                self.validity.push(false);
            }
        }
    }
}

impl<P: Primitive> Scalar for OptionalFixedList<P> {
    type Ref<'slice> = OptionalFixedSlice<'slice, P>;
    type Mut<'slice> = OptionalFixedSliceMut<'slice, P>;

    #[inline]
    fn as_ref(&self) -> Self::Ref<'_> {
        OptionalFixedSlice::new(&self.validity, &self.data[..])
    }

    #[inline]
    fn as_mut(&mut self) -> Self::Mut<'_> {
        OptionalFixedSliceMut::new(&mut self.validity, &mut self.data[..])
    }
}

impl<P: Primitive> From<Vec<Option<P>>> for OptionalFixedList<P> {
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
pub struct OptionalFixedSlice<'slice, P: Primitive> {
    pub(crate) validity: &'slice BitSlice,
    pub(crate) data: &'slice [P],
}

impl<'slice, P: Primitive> ScalarRef<'slice> for OptionalFixedSlice<'slice, P> {
    type Owned = OptionalFixedList<P>;

    #[inline]
    fn to_owned(self) -> Self::Owned {
        OptionalFixedList {
            data: ToOwned::to_owned(self.data),
            validity: self.validity.to_owned(),
        }
    }
}

impl<P: Primitive> Clone for OptionalFixedSlice<'_, P> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            validity: self.validity,
            data: self.data,
        }
    }
}

impl<'slice, P: Primitive> OptionalFixedSlice<'slice, P> {
    pub(crate) fn new(validity: &'slice BitSlice, data: &'slice [P]) -> Self {
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
    pub fn slice(&self, range: Range<usize>) -> OptionalFixedSlice<'_, P> {
        Self {
            validity: &self.validity[range.clone()],
            data: &self.data[range],
        }
    }

    #[inline]
    pub fn slice_from(&self, range: RangeFrom<usize>) -> OptionalFixedSlice<'_, P> {
        OptionalFixedSlice {
            validity: &self.validity[range.clone()],
            data: &self.data[range],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'slice, P: Primitive> IntoIterator for OptionalFixedSlice<'slice, P> {
    type Item = Option<&'slice P>;
    type IntoIter = OptionalFixedSizeSliceIter<'slice, P>;

    fn into_iter(self) -> Self::IntoIter {
        OptionalFixedSizeSliceIter {
            slice: self,
            pos: 0,
        }
    }
}

pub struct OptionalFixedSizeSliceIter<'slice, P: Primitive> {
    slice: OptionalFixedSlice<'slice, P>,
    pos: usize,
}

impl<'slice, P: Primitive> Iterator for OptionalFixedSizeSliceIter<'slice, P> {
    type Item = Option<&'slice P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.slice.len() {
            return None;
        }
        let pos = self.pos;
        self.pos += 1;
        Some(if unsafe { *self.slice.validity.get_unchecked(pos) } {
            Some(unsafe { self.slice.data.get_unchecked(pos) })
        } else {
            None
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct OptionalFixedSliceMut<'slice, P: Primitive> {
    pub(crate) validity: &'slice mut BitSlice,
    pub(crate) data: &'slice mut [P],
}

impl<'slice, P: Primitive> ScalarMut<'slice> for OptionalFixedSliceMut<'slice, P> {
    type Owned = OptionalFixedList<P>;

    #[inline]
    fn to_owned(self) -> Self::Owned {
        OptionalFixedList {
            validity: self.validity.to_owned(),
            data: self.data.to_owned(),
        }
    }
}

impl<'slice, P: Primitive> OptionalFixedSliceMut<'slice, P> {
    pub(crate) fn new(validity: &'slice mut BitSlice, data: &'slice mut [P]) -> Self {
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

    #[inline]
    pub fn slice_from(&self, range: RangeFrom<usize>) -> OptionalFixedSlice<'_, P> {
        OptionalFixedSlice {
            validity: &self.validity[range.clone()],
            data: &self.data[range],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }
}
