use common::array::{
    Array, ConstFixedSizedListArray, IdArray, IdentifiedArray, ListArray,
    NullableFixedSizedListArray, PrimitiveArray,
};

use super::index::IndexImpl;

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

pub type StringLabel = IdArray<ListArray<u8>>;
pub type IPv4Label = IdArray<ConstFixedSizedListArray<u8, 4>>;
pub type IPv6Label = IdArray<ConstFixedSizedListArray<u8, 16>>;
pub type IntLabel = IdArray<PrimitiveArray<i64>>;
pub type BoolLabel = IdArray<PrimitiveArray<bool>>;

#[derive(Debug, Clone)]
pub struct LabelColumn<A: IdentifiedArray> {
    array: A,
    #[allow(unused)]
    index: Vec<IndexImpl<A::ID>>,
}

impl<A: IdentifiedArray> PartialEq for LabelColumn<A>
where
    for<'a> A::ItemRef<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.array.iter().eq(other.array.iter())
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

pub type LabelImpl = common::column::Label<
    LabelColumn<StringLabel>,
    LabelColumn<IPv4Label>,
    LabelColumn<IPv6Label>,
    LabelColumn<IntLabel>,
    LabelColumn<BoolLabel>,
>;

pub type FieldImpl = common::column::Field<
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
