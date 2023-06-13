use bitvec::prelude::*;

use super::Array;
use crate::{
    primitive::Primitive,
    scalar::list::{NfsList, NfsSlice, NfsSliceMut},
};

#[derive(Debug, Clone)]
pub struct FixedListArray<P: Primitive> {
    pub(crate) data: Vec<P>,
    pub(crate) list_size: u32,
}

impl<P: Primitive> FixedListArray<P> {
    #[inline]
    pub fn new(list_size: u32) -> Self {
        Self {
            list_size,
            data: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize, list_size: u32) -> Self {
        Self {
            list_size,
            data: Vec::with_capacity(capacity * list_size as usize),
        }
    }

    #[inline]
    pub(crate) unsafe fn slice_raw_mut(&mut self, start: usize, end: usize) -> &mut [P] {
        self.data.get_unchecked_mut(start..end)
    }

    #[inline]
    pub(crate) unsafe fn slice_raw(&self, start: usize, end: usize) -> &[P] {
        self.data.get_unchecked(start..end)
    }

    #[inline]
    pub fn list_size(&self) -> usize {
        self.list_size as _
    }
}

impl<P: Primitive> Array for FixedListArray<P> {
    type Item = Vec<P>;
    type ItemRef<'a> = &'a [P];
    type ItemRefMut<'a> = &'a mut [P];

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        if id * self.list_size() > self.data.len() {
            None
        } else {
            Some(unsafe { self.get_unchecked(id) })
        }
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        self.slice_raw(id * self.list_size(), (id + 1) * self.list_size())
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        if id * self.list_size() > self.data.len() {
            None
        } else {
            Some(unsafe { self.slice_raw_mut(id * self.list_size(), (id + 1) * self.list_size()) })
        }
    }

    #[inline]
    fn push(&mut self, value: Self::Item) {
        self.data.extend_from_slice(&value);
    }

    #[inline]
    fn push_zero(&mut self) {
        self.push(vec![Default::default(); self.list_size()]);
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len() / self.list_size()
    }
}

#[derive(Debug, Clone)]
pub struct ConstFixedListArray<P: Primitive, const SIZE: usize> {
    data: Vec<P>,
}

impl<P: Primitive, const SIZE: usize> Default for ConstFixedListArray<P, SIZE> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Primitive, const SIZE: usize> ConstFixedListArray<P, SIZE> {
    #[inline]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity * SIZE),
        }
    }
}

impl<P: Primitive, const SIZE: usize> Array for ConstFixedListArray<P, SIZE> {
    type Item = [P; SIZE];
    type ItemRef<'a> = &'a [P; SIZE];
    type ItemRefMut<'a> = &'a mut [P; SIZE];

    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        if (id + 1) * SIZE > self.data.len() {
            return None;
        }
        Some(unsafe { self.get_unchecked(id) })
    }

    unsafe fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        self.data
            .get_unchecked(id * SIZE..(id + 1) * SIZE)
            .split_array_ref::<SIZE>()
            .0
    }

    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        if (id + 1) * SIZE > self.data.len() {
            return None;
        }

        Some(
            self.data[id * SIZE..(id + 1) * SIZE]
                .split_array_mut::<SIZE>()
                .0,
        )
    }

    fn push(&mut self, value: Self::Item) {
        self.data.extend_from_slice(&value[..])
    }

    fn push_zero(&mut self) {
        self.data.extend_from_slice(&[P::default(); SIZE][..])
    }

    fn len(&self) -> usize {
        self.data.len() / SIZE
    }
}

#[derive(Debug, Clone)]
pub struct NullableFixedListArray<P: Primitive> {
    validity: BitVec,
    data: FixedListArray<P>,
}

impl<P: Primitive> NullableFixedListArray<P> {
    #[inline]
    pub fn new(list_size: u32) -> Self {
        Self {
            data: FixedListArray::<P>::new(list_size),
            validity: BitVec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize, list_size: u32) -> Self {
        Self {
            data: FixedListArray::<P>::with_capacity(capacity, list_size),
            validity: BitVec::new(),
        }
    }

    #[inline]
    pub fn list_size(&self) -> usize {
        self.data.list_size()
    }
}

impl<P: Primitive> Array for NullableFixedListArray<P> {
    type Item = NfsList<P>;
    type ItemRef<'a> = NfsSlice<'a, P>;
    type ItemRefMut<'a> = NfsSliceMut<'a, P>;

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        if id * self.data.list_size() > self.data.data.len() {
            None
        } else {
            Some(unsafe { self.get_unchecked(id) })
        }
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        NfsSlice {
            validity: &self.validity[id * self.data.list_size()..(id + 1) * self.data.list_size()],
            data: self
                .data
                .slice_raw(id * self.data.list_size(), (id + 1) * self.data.list_size()),
        }
    }

    #[inline]
    fn get_mut(&mut self, offset: usize) -> Option<Self::ItemRefMut<'_>> {
        let (start, end) = (
            offset * self.data.list_size(),
            (offset + 1) * self.data.list_size(),
        );
        Some(NfsSliceMut::new(&mut self.validity[start..end], unsafe {
            self.data.slice_raw_mut(start, end)
        }))
    }

    #[inline]
    fn push(&mut self, value: Self::Item) {
        self.validity.extend(value.validity);
        self.data.data.extend(value.data);
    }

    #[inline]
    fn push_zero(&mut self) {
        for _ in 0..self.data.list_size {
            self.validity.push(false);
        }
        self.data.push_zero();
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}
