#[derive(Debug, PartialEq, Clone)]
pub enum Index<Inverted, Sparse> {
    Inverted(Inverted),
    Sparse(Sparse),
}
