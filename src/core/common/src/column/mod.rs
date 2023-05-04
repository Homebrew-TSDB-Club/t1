pub mod field;
pub mod label;

#[derive(Debug)]
pub enum ColumnType {
    Label,
    Field,
}
