use std::hash::Hash;

use common::{
    array::{
        Array, ConstFixedSizedListArray, IdArray, ListArray, NullableFixedSizedListArray,
        PrimitiveArray,
    },
    column::{Field, Label},
};
use croaring::Bitmap;

use super::index::{IndexImpl, IndexType, Set};

pub type UInt8Field = NullableFixedSizedListArray<u8>;
pub type UInt16Field = NullableFixedSizedListArray<u16>;
pub type UInt32Field = NullableFixedSizedListArray<u32>;
pub type UInt64Field = NullableFixedSizedListArray<u64>;
pub type Int8Field = NullableFixedSizedListArray<i8>;
pub type Int16Field = NullableFixedSizedListArray<i16>;
pub type Int32Field = NullableFixedSizedListArray<i32>;
pub type Int64Field = NullableFixedSizedListArray<i64>;
pub type Float32Field = NullableFixedSizedListArray<f32>;
pub type Float64Field = NullableFixedSizedListArray<f64>;
pub type BoolField = NullableFixedSizedListArray<bool>;

pub type StringLabel = ListArray<u8>;
pub type IPv4Label = ConstFixedSizedListArray<u8, 4>;
pub type IPv6Label = ConstFixedSizedListArray<u8, 16>;
pub type IntLabel = PrimitiveArray<i64>;
pub type BoolLabel = PrimitiveArray<bool>;

#[derive(Debug, Clone)]
pub struct LabelColumn<A> {
    array: IdArray<A>,
    index: IndexImpl<usize>,
}

impl<A: Array + Default> LabelColumn<A> {
    pub fn new(index: IndexType<(), u32>) -> Self {
        Self {
            array: IdArray::<A>::new(A::default()),
            index: IndexImpl::new(index),
        }
    }
}

impl<A: Array> PartialEq for LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.array.iter().eq(other.array.iter())
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    pub fn lookup<'s: 'value, 'value>(
        &'s self,
        value: Option<A::ItemRef<'value>>,
        superset: &mut Set,
    ) {
        let id = value
            .as_ref()
            .map(|v| self.array.lookup_id(v))
            .unwrap_or(Some(0));
        match id {
            Some(id) => self.index.lookup(&id, superset),
            None => superset.clear(),
        }

        if !self.index.exactly() {
            match superset {
                Set::Universe => {
                    let matched: Bitmap = self
                        .array
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| item == &value)
                        .map(|(id, _)| id as u32)
                        .collect();
                    superset.and_inplace(Set::Some(matched));
                }
                Set::Some(set) => {
                    *set = set
                        .iter()
                        .filter(|item| self.array.get_unchecked(*item as usize) == value)
                        .collect()
                }
            }
        }
    }

    pub fn push(&mut self, value: Option<A::ItemRef<'_>>) {
        let id = self.array.push_and_get_id(value);
        self.index.insert(self.array.len() - 1, id);
    }
}

#[derive(Debug, Clone)]
pub struct FieldColumn<A: Array> {
    array: A,
}

impl<A: Array> PartialEq for FieldColumn<A>
where
    for<'a> A::ItemRef<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.array.iter().eq(other.array.iter())
    }
}

pub type LabelImpl = Label<
    LabelColumn<StringLabel>,
    LabelColumn<IPv4Label>,
    LabelColumn<IPv6Label>,
    LabelColumn<IntLabel>,
    LabelColumn<BoolLabel>,
>;

pub type FieldImpl = Field<
    FieldColumn<UInt8Field>,
    FieldColumn<UInt16Field>,
    FieldColumn<UInt32Field>,
    FieldColumn<UInt64Field>,
    FieldColumn<Int8Field>,
    FieldColumn<Int16Field>,
    FieldColumn<Int32Field>,
    FieldColumn<Int64Field>,
    FieldColumn<Float32Field>,
    FieldColumn<Float64Field>,
    FieldColumn<BoolField>,
>;

#[cfg(test)]
mod tests {
    use super::{LabelColumn, StringLabel};
    use crate::mutable::index::{IndexType, Set};

    #[test]
    fn label_lookup_value() {
        let mut column = LabelColumn::<StringLabel>::new(IndexType::Inverted(()));
        column.push(Some("test".as_ref()));
        column.push(None);
        column.push(Some("hello".as_ref()));
        column.push(Some("world".as_ref()));
        column.push(Some("hello".as_ref()));

        let mut result = Set::Universe;
        column.lookup(Some("hello".as_ref()), &mut result);
        assert_eq!(result, Set::Some([2_u32, 4_u32].into_iter().collect()));

        let mut result = Set::Universe;
        column.lookup(Some("universe".as_ref()), &mut result);
        assert_eq!(result, Set::Some([].into_iter().collect()));

        let mut result = Set::Universe;
        column.lookup(None, &mut result);
        assert_eq!(result, Set::Some([1_u32].into_iter().collect()));
    }
}
