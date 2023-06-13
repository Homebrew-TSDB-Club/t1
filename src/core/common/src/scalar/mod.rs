pub mod array;
pub mod list;
pub mod option;
pub mod primitive;
pub mod vec;

pub trait Scalar: 'static + Clone + Sized {
    type Ref<'a>: ScalarRef<'a>
    where
        Self: 'a;
    type RefMut<'a>: ScalarMut<'a>
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Ref<'_>;
}

pub trait ScalarRef<'a>: Clone {
    type Owned: Scalar;

    fn to_owned(self) -> Self::Owned;
}

pub trait ScalarMut<'a> {
    type Owned: Scalar;
}

#[cfg(test)]
mod tests {
    use crate::scalar::{list::NfsList, Scalar};

    #[test]
    fn test_list() {
        let list = NfsList::from(vec![None, Some(1)]);
        assert_eq!(list.as_ref().get(0), Some(None));
        assert_eq!(list.as_ref().get(1).map(|s| s.cloned()), Some(Some(1)));
        assert_eq!(list.as_ref().get(2), None);
    }
}
