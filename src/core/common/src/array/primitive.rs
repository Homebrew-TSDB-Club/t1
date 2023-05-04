use super::Array;
use crate::primitive::Primitive;

#[derive(Debug, Clone)]
pub struct PrimitiveArray<P: Primitive> {
    data: Vec<P>,
}

impl<P: Primitive> Default for PrimitiveArray<P> {
    #[inline]
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<P: Primitive> PrimitiveArray<P> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }
}

impl<P: Primitive> Array for PrimitiveArray<P> {
    type Item = P;
    type ItemRef<'a> = &'a P;
    type ItemRefMut<'a> = &'a mut P;

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        self.data.get(id)
    }

    #[inline]
    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        unsafe { self.data.get_unchecked(id) }
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        self.data.get_mut(id)
    }

    #[inline]
    fn push(&mut self, value: Self::Item) {
        self.data.push(value)
    }

    #[inline]
    fn push_zero(&mut self) {
        self.data.push(P::default())
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}
