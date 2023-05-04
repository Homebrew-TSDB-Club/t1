use std::ops::Range;

use common::{
    array::{fixed::NullableFixedListArray, Array, ArrayIterator},
    column::field::{Field, FieldValue},
    scalar::ScalarRef,
};
use croaring::Bitmap;
use paste::paste;

pub type UInt8Field = NullableFixedListArray<u8>;
pub type UInt16Field = NullableFixedListArray<u16>;
pub type UInt32Field = NullableFixedListArray<u32>;
pub type UInt64Field = NullableFixedListArray<u64>;
pub type Int8Field = NullableFixedListArray<i8>;
pub type Int16Field = NullableFixedListArray<i16>;
pub type Int32Field = NullableFixedListArray<i32>;
pub type Int64Field = NullableFixedListArray<i64>;
pub type Float32Field = NullableFixedListArray<f32>;
pub type Float64Field = NullableFixedListArray<f64>;
pub type BoolField = NullableFixedListArray<bool>;

pub type ArrayImpl = Field<
    UInt8Field,
    UInt16Field,
    UInt32Field,
    UInt64Field,
    Int8Field,
    Int16Field,
    Int32Field,
    Int64Field,
    Float32Field,
    Float64Field,
    BoolField,
>;

#[derive(Debug)]
pub struct FieldImpl(ArrayImpl);

impl From<ArrayImpl> for FieldImpl {
    #[inline]
    fn from(value: ArrayImpl) -> Self {
        Self(value)
    }
}

impl FieldImpl {
    #[inline]
    pub fn push_zero(&mut self) {
        macro_rules! push {
            ($($label_type:ident), *) => {
                paste! {
                match &mut self.0 {
                    $(Field::$label_type(column) => column.push_zero(),)*
                }
                }
            };
        }

        push!(UInt8, UInt16, UInt32, UInt64, Int8, Int16, Int32, Int64, Float32, Float64, Bool);
    }

    #[inline]
    pub fn map(&self, row_set: &Bitmap, range: Range<usize>) -> Self {
        macro_rules! map {
            ($($field_type:ident), *) => {
                paste! {
                match &self.0 {
                    $(
                    Field::$field_type(column) => {
                        let mut field = [<$field_type Field>]::with_capacity(
                            row_set.cardinality() as usize,
                            column.list_size() as u32,
                        );
                        for item in column.iter() {
                            field.push(ScalarRef::to_owned(item.slice(range.clone())));
                        }
                        Field::$field_type(field)
                    }
                    )*
                }.into()
                }
            };
        }

        map!(UInt8, UInt16, UInt32, UInt64, Int8, Int16, Int32, Int64, Float32, Float64, Bool)
    }

    #[inline]
    pub fn len(&self) -> usize {
        macro_rules! len {
            ($($field_type:ident), *) => {
                paste! {
                match &self.0 {
                    $(Field::$field_type(column) => column.len(),)*
                }.into()
                }
            };
        }

        len!(UInt8, UInt16, UInt32, UInt64, Int8, Int16, Int32, Int64, Float32, Float64, Bool)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> FieldIter<'_> {
        FieldIter(match &self.0 {
            Field::UInt8(field) => Field::UInt8(field.iter()),
            Field::UInt16(field) => Field::UInt16(field.iter()),
            Field::UInt32(field) => Field::UInt32(field.iter()),
            Field::UInt64(field) => Field::UInt64(field.iter()),
            Field::Int8(field) => Field::Int8(field.iter()),
            Field::Int16(field) => Field::Int16(field.iter()),
            Field::Int32(field) => Field::Int32(field.iter()),
            Field::Int64(field) => Field::Int64(field.iter()),
            Field::Float32(field) => Field::Float32(field.iter()),
            Field::Float64(field) => Field::Float64(field.iter()),
            Field::Bool(field) => Field::Bool(field.iter()),
        })
    }
}

pub struct FieldIter<'a>(
    Field<
        ArrayIterator<'a, UInt8Field>,
        ArrayIterator<'a, UInt16Field>,
        ArrayIterator<'a, UInt32Field>,
        ArrayIterator<'a, UInt64Field>,
        ArrayIterator<'a, Int8Field>,
        ArrayIterator<'a, Int16Field>,
        ArrayIterator<'a, Int32Field>,
        ArrayIterator<'a, Int64Field>,
        ArrayIterator<'a, Float32Field>,
        ArrayIterator<'a, Float64Field>,
        ArrayIterator<'a, BoolField>,
    >,
);

impl<'a> Iterator for FieldIter<'a> {
    type Item = FieldValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Field::UInt8(field) => field.next().map(Field::UInt8),
            Field::UInt16(field) => field.next().map(Field::UInt16),
            Field::UInt32(field) => field.next().map(Field::UInt32),
            Field::UInt64(field) => field.next().map(Field::UInt64),
            Field::Int8(field) => field.next().map(Field::Int8),
            Field::Int16(field) => field.next().map(Field::Int16),
            Field::Int32(field) => field.next().map(Field::Int32),
            Field::Int64(field) => field.next().map(Field::Int64),
            Field::Float32(field) => field.next().map(Field::Float32),
            Field::Float64(field) => field.next().map(Field::Float64),
            Field::Bool(field) => field.next().map(Field::Bool),
        }
    }
}
