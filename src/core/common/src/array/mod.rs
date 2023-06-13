pub(crate) mod dictionary;
pub mod fixed;
pub mod id;
pub mod list;
pub mod primitive;

use crate::scalar::{Scalar, ScalarMut, ScalarRef};

pub trait Array: 'static + Sized {
    type Item: for<'a> Scalar<Ref<'a> = Self::ItemRef<'a>, RefMut<'a> = Self::ItemRefMut<'a>>;
    type ItemRef<'a>: ScalarRef<'a, Owned = Self::Item>
    where
        Self: 'a;
    type ItemRefMut<'a>: ScalarMut<'a, Owned = Self::Item>
    where
        Self: 'a;

    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>>;
    unsafe fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_>;
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>>;
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
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug)]
pub struct ArrayIterator<'a, A: Array> {
    array: &'a A,
    pos: usize,
}

impl<'a, A: Array> Iterator for ArrayIterator<'a, A> {
    type Item = A::ItemRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.array.len() {
            None
        } else {
            let item = unsafe { self.array.get_unchecked(self.pos) };
            self.pos += 1;
            Some(item)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fixed::{FixedListArray, NullableFixedListArray},
        id::IdArray,
        list::ListArray,
        primitive::PrimitiveArray,
        Array,
    };
    use crate::scalar::list::NfsList;

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
        let mut array = NullableFixedListArray::new(2);
        array.push(NfsList::<_>::from(vec![None, Some(1)]));
        array.push(NfsList::<_>::from(vec![Some(2), Some(3)]));
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
