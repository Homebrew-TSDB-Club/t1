use std::{error::Error, process::Output};

// use crate::Source;

// pub trait Execution: Stream {
//     type Output<'s>
//     where
//         Self: 's;
//     type Error: Error;
//     type Stream<'s>: Stream<Output<'s> = Result<<Self as Execution>::Output<'s>, Self::Error>>
//     where
//         Self: 's;

//     fn execute(&self) -> Self::Stream<'_>;
// }

pub enum ExecutionImpl {}

// impl Source for Table {
//     type Execution;

//     type ScanFut<'future>
//     where
//         Self: 'future;

//     fn scan(
//         &self,
//         context: &crate::Context,
//         projection: &[usize],
//         filter: &[common::expression::Matcher<usize>],
//     ) -> Self::ScanFut<'_> {
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {}
