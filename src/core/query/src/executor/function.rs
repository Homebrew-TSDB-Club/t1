use chunk::mutable::Records;
use common::column::field::Field;

use super::DynError;

pub trait Function {
    fn call(&self, records: Records) -> Result<Records, DynError>;
}

pub struct Rate {
    field_id: usize,
}

impl Function for Rate {
    fn call(&self, records: Records) -> Result<Records, DynError> {
        for value in records.fields[self.field_id].iter() {
            match value {
                Field::UInt8(value) => {
                    value.into_iter();
                }
                Field::UInt16(value) => todo!(),
                Field::UInt32(value) => todo!(),
                Field::UInt64(value) => todo!(),
                Field::Int8(value) => todo!(),
                Field::Int16(value) => todo!(),
                Field::Int32(value) => todo!(),
                Field::Int64(value) => todo!(),
                Field::Float32(value) => todo!(),
                Field::Float64(value) => todo!(),
                Field::Bool(value) => todo!(),
            }
        }
        todo!()
    }
}
