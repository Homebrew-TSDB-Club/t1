pub mod expression;
pub mod promql;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("parsing wrong: {}", .err)]
    ParsingWrong { err: String },
    #[error("query does not have a metric name")]
    NoName,
}
