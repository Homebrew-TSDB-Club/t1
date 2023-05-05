pub mod expression;
pub mod promql;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("parser internal error: {:?}", .err)]
    InternalError { err: String },
    #[error("query does not have a metric name")]
    NoName,
}
