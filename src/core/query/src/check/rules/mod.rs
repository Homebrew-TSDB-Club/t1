mod r#type;

use common::column::ColumnType;
use thiserror::Error;

use super::Env;

#[derive(Error, Debug)]
#[error("{place} type mismatch, expect {expect} found {found}")]
pub struct TypeMismatch {
    place: String,
    expect: ColumnType,
    found: ColumnType,
}

pub(crate) trait Rule {
    fn check(&self, env: &Env) -> Result<(), TypeMismatch>;
}
