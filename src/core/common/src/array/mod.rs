pub(crate) mod dictionary;
pub mod fixed;
pub mod id;
pub mod list;
pub mod primitive;

use std::mem::transmute;

use crate::scalar::{Scalar, ScalarMut, ScalarRef};

pub trait Array: 'static + Sized {
    type Item: for<'iter> Scalar<
        Ref<'iter> = Self::ItemRef<'iter>,
        Mut<'iter> = Self::ItemMut<'iter>,
    >;
    type ItemRef<'iter>: ScalarRef<'iter, Owned = Self::Item>
    where
        Self: 'iter;
    type ItemMut<'iter>: ScalarMut<'iter, Owned = Self::Item>
    where
        Self: 'iter;

    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>>;
    #[allow(clippy::missing_safety_doc)]
    unsafe fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_>;
    #[allow(clippy::missing_safety_doc)]
    unsafe fn get_unchecked_mut(&mut self, id: usize) -> Self::ItemMut<'_>;
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemMut<'_>>;
    fn push(&mut self, value: Self::Item);
    fn push_zero(&mut self);
    fn len(&self) -> usize;

    #[inline]
    fn iter(&self) -> ArrayIterator<'_, Self> {
        ArrayIterator {
            array: self,
            pos: 0,
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> ArrayMutIterator<'_, Self> {
        ArrayMutIterator {
            array: self,
            pos: 0,
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug)]
pub struct ArrayIterator<'iter, A: Array> {
    array: &'iter A,
    pos: usize,
}

impl<'iter, A: Array> Iterator for ArrayIterator<'iter, A> {
    type Item = A::ItemRef<'iter>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.array.len() {
            return None;
        }
        let item = unsafe { self.array.get_unchecked(self.pos) };
        self.pos += 1;
        Some(item)
    }
}

#[derive(Debug)]
pub struct ArrayMutIterator<'iter, A: Array> {
    array: &'iter mut A,
    pos: usize,
}

impl<'iter, A: Array> Iterator for ArrayMutIterator<'iter, A> {
    type Item = A::ItemMut<'iter>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.array.len() {
            return None;
        }
        let item = unsafe { transmute(self.array.get_unchecked_mut(self.pos)) };
        self.pos += 1;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fixed::{FixedListArray, OptionalFixedListArray},
        id::IdArray,
        list::ListArray,
        primitive::PrimitiveArray,
        Array,
    };
    use crate::scalar::list::OptionalFixedList;

    #[test]
    fn test_fixed_sized_list_array() {
        let mut array = FixedListArray::new(2);
        array.push(vec![1, 2]);
        array.push(vec![3, 4]);
        assert_eq!(array.get(0), Some([1, 2].as_slice()));
        assert_eq!(array.get(1), Some([3, 4].as_slice()));
        array.push_zero();
        assert_eq!(array.get(2), Some([0; 2].as_slice()));
    }

    #[test]
    fn test_list_array() {
        let mut array = ListArray::new();
        array.push(vec![1, 2]);
        array.push(vec![2, 3, 4]);
        assert_eq!(array.get(0), Some([1, 2].as_slice()));
        assert_eq!(array.get(1), Some([2, 3, 4].as_slice()));
    }

    #[test]
    fn test_nullable_array() {
        let mut array = OptionalFixedListArray::new(2);
        array.push(OptionalFixedList::<_>::from(vec![None, Some(1)]));
        array.push(OptionalFixedList::<_>::from(vec![Some(2), Some(3)]));
        assert_eq!(array.get(0).unwrap().get(0), Some(None));
        assert_eq!(array.get(0).unwrap().get(1), Some(Some(&1)));
        assert_eq!(array.get(1).unwrap().get(0), Some(Some(&2)));
        assert_eq!(array.get(1).unwrap().get(1), Some(Some(&3)));
        let mut ref_mut = array.get_mut(0).unwrap();
        ref_mut.set(0, Some(1));
        assert_eq!(array.get(0).unwrap().get(0), Some(Some(&1)));
    }

    #[test]
    fn test_id_array() {
        let mut array = IdArray::<ListArray<u8>>::new(ListArray::<u8>::new());
        array.push(Some(Vec::from("foo")));
        array.push(Some(Vec::from("bar")));
        array.push(Some(Vec::from("quaz")));
        array.push(Some(Vec::from("bar")));
        assert_eq!(array.get(0), Some(Some("foo".as_ref())));
        assert_eq!(array.get(1), Some(Some("bar".as_ref())));
        assert_eq!(array.get(2), Some(Some("quaz".as_ref())));
        assert_eq!(
            array.get(3).unwrap().unwrap().as_ptr(),
            array.get(1).unwrap().unwrap().as_ptr()
        );
    }

    #[test]
    fn test_primitive_array() {
        let mut array = PrimitiveArray::new();
        array.push(1);
        array.push(2);
        array.push(3);
        assert_eq!(array.get(0), Some(&1));
        assert_eq!(array.get(1), Some(&2));
        assert_eq!(array.get(2), Some(&3));
    }
}
