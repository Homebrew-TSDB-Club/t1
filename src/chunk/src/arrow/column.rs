use arrow2::array::PrimitiveArray;
use common::array::{dictionary::Dictionary, Array};

pub type UInt8Field = PrimitiveArray<u8>;
pub type UInt16Field = PrimitiveArray<u16>;
pub type UInt32Field = PrimitiveArray<u32>;
pub type UInt64Field = PrimitiveArray<u64>;
pub type Int8Field = PrimitiveArray<i8>;
pub type Int16Field = PrimitiveArray<i16>;
pub type Int32Field = PrimitiveArray<i32>;
pub type Int64Field = PrimitiveArray<i64>;
pub type Float32Field = PrimitiveArray<f32>;
pub type Float64Field = PrimitiveArray<f64>;
pub type BoolField = PrimitiveArray<bool>;
