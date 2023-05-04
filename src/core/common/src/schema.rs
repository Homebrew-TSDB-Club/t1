use crate::{
    column::{field::FieldType, label::LabelType},
    index::Index,
};

#[derive(Debug)]
pub struct Label {
    pub r#type: LabelType,
    pub name: String,
}

#[derive(Debug)]
pub struct Field {
    pub r#type: FieldType,
    pub name: String,
}

#[derive(Debug)]
pub struct Schema {
    pub labels: Vec<Label>,
    pub fields: Vec<Field>,
    pub index: Vec<Index<(), u32>>,
}
