use executor::Stream;

pub trait Execution {
    type Stream: Stream;

    fn execute(&self) -> Self::Stream;
}

pub enum ExecutionImpl {}

#[cfg(test)]
mod tests {}
