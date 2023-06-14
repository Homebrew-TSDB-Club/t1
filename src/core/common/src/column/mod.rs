use std::fmt::{self, Display, Formatter};

use self::{field::FieldType, label::LabelType};

pub mod field;
pub mod label;

#[derive(Debug)]
pub enum ColumnType {
    Label(LabelType),
    Field(FieldType),
}

impl Display for ColumnType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Label(label) => label.fmt(f),
            Self::Field(field) => field.fmt(f),
        }
    }
}
