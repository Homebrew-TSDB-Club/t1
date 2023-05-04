use crate::scalar::list::NfsSlice;

#[derive(Debug, Clone, PartialEq)]
pub enum Field<U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, B> {
    UInt8(U8),
    UInt16(U16),
    UInt32(U32),
    UInt64(U64),
    Int8(I8),
    Int16(I16),
    Int32(I32),
    Int64(I64),
    Float32(F32),
    Float64(F64),
    Bool(B),
}

pub type FieldType = Field<(), (), (), (), (), (), (), (), (), (), ()>;

pub type FieldValue<'a> = Field<
    NfsSlice<'a, u8>,
    NfsSlice<'a, u16>,
    NfsSlice<'a, u32>,
    NfsSlice<'a, u64>,
    NfsSlice<'a, i8>,
    NfsSlice<'a, i16>,
    NfsSlice<'a, i32>,
    NfsSlice<'a, i64>,
    NfsSlice<'a, f32>,
    NfsSlice<'a, f64>,
    NfsSlice<'a, bool>,
>;
