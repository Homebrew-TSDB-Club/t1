use std::ops::Range;

use common::{
    array::{fixed::OptionalFixedListArray, Array},
    column::field::Field,
    context::Context,
    scalar::{list::OptionalFixedList, ScalarRef},
    try_yield,
};
use croaring::Bitmap;
use paste::paste;

pub type UInt8Field = OptionalFixedListArray<u8>;
pub type UInt16Field = OptionalFixedListArray<u16>;
pub type UInt32Field = OptionalFixedListArray<u32>;
pub type UInt64Field = OptionalFixedListArray<u64>;
pub type Int8Field = OptionalFixedListArray<i8>;
pub type Int16Field = OptionalFixedListArray<i16>;
pub type Int32Field = OptionalFixedListArray<i32>;
pub type Int64Field = OptionalFixedListArray<i64>;
pub type Float32Field = OptionalFixedListArray<f32>;
pub type Float64Field = OptionalFixedListArray<f64>;
pub type BoolField = OptionalFixedListArray<bool>;

type ArrayImpl = Field<
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
#[repr(transparent)]
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
    pub async fn map(&self, cx: &mut Context, row_set: &Bitmap, range: Range<usize>) -> Self {
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

                        let mut iter = column.iter();
                        while let Some(item) = iter.next() {
                            field.push(ScalarRef::to_owned(item.slice(range.clone())));
                            try_yield!(cx);
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
}

pub type FieldItemImpl = Field<
    OptionalFixedList<u8>,
    OptionalFixedList<u16>,
    OptionalFixedList<u32>,
    OptionalFixedList<u64>,
    OptionalFixedList<i8>,
    OptionalFixedList<i16>,
    OptionalFixedList<i32>,
    OptionalFixedList<i64>,
    OptionalFixedList<f32>,
    OptionalFixedList<f64>,
    OptionalFixedList<bool>,
>;
