use super::Array;
use crate::primitive::Primitive;

#[derive(Debug, Clone)]
pub struct ListArray<P: Primitive> {
    data: Vec<P>,
    offsets: Vec<usize>,
}

impl<P: Primitive> ListArray<P> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::<P>::with_capacity(capacity),
            offsets: vec![0],
        }
    }
}

impl<P: Primitive> Default for ListArray<P> {
    #[inline]
    fn default() -> Self {
        Self {
            data: Vec::<P>::new(),
            offsets: vec![0],
        }
    }
}

impl<P: Primitive> Array for ListArray<P> {
    type Item = Vec<P>;
    type ItemRef<'a> = &'a [P];
    type ItemRefMut<'a> = &'a mut [P];

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        let offset = self.offsets.get(id)?;
        let end = self.offsets.get(id + 1)?;
        Some(&self.data[*offset..*end])
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        let offset = self.offsets[id];
        let end = self.offsets[id + 1];
        &self.data.get_unchecked(offset..end)
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        let offset = self.offsets.get(id)?;
        let end = self.offsets.get(id + 1)?;
        Some(&mut self.data[*offset..*end])
    }

    #[inline]
    fn push(&mut self, value: Self::Item) {
        let id = self.offsets.len() - 1;
        let end = self.offsets[id] + value.len();
        self.offsets.push(end);
        self.data.extend_from_slice(&value);
    }

    #[inline]
    fn push_zero(&mut self) {
        self.offsets.push(self.offsets[self.offsets.len() - 1]);
    }

    #[inline]
    fn len(&self) -> usize {
        self.offsets.len() - 1
    }
}
