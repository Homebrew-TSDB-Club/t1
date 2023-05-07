pub type LabelType = Label<(), (), (), (), ()>;
pub type FieldType = Field<(), (), (), (), (), (), (), (), (), (), ()>;

#[derive(Debug, Clone, PartialEq)]
pub enum Label<S, IP4, IP6, I, B> {
    String(S),
    IPv4(IP4),
    IPv6(IP6),
    Int(I),
    Bool(B),
}

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
