use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
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

type TypeInner = Field<(), (), (), (), (), (), (), (), (), (), ()>;

#[derive(Clone)]
pub struct FieldType(TypeInner);

impl From<TypeInner> for FieldType {
    #[inline]
    fn from(value: TypeInner) -> Self {
        Self(value)
    }
}

impl AsRef<TypeInner> for FieldType {
    fn as_ref(&self) -> &TypeInner {
        &self.0
    }
}

impl Debug for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.0 {
            Field::UInt8(_) => "uint8",
            Field::UInt16(_) => "uint16",
            Field::UInt32(_) => "uint32",
            Field::UInt64(_) => "uint64",
            Field::Int8(_) => "int8",
            Field::Int16(_) => "int16",
            Field::Int32(_) => "int32",
            Field::Int64(_) => "int64",
            Field::Float32(_) => "float32",
            Field::Float64(_) => "float64",
            Field::Bool(_) => "bool",
        };
        f.write_str(s)
    }
}

impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
