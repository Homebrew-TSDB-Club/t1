pub mod field;
pub mod label;

use common::column::label::LabelTypeMismatch;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("regex match only supports string label")]
    RegexStringOnly,
    #[error("regex pattern error: {}", source)]
    PatternError {
        #[from]
        source: regex::Error,
    },
    #[error(transparent)]
    MismatchType {
        #[from]
        source: LabelTypeMismatch,
    },
}
