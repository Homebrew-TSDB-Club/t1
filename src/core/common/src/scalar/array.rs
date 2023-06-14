use super::{Scalar, ScalarMut, ScalarRef};
use crate::primitive::Primitive;

impl<P: Primitive, const SIZE: usize> Scalar for [P; SIZE] {
    type Ref<'a> = &'a [P; SIZE]
    where
        Self: 'a;

    type Mut<'a> = &'a mut [P; SIZE]
    where
        Self: 'a;

    #[inline]
    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }

    #[inline]
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}

impl<'a, P: Primitive, const SIZE: usize> ScalarRef<'a> for &'a [P; SIZE] {
    type Owned = [P; SIZE];

    #[inline]
    fn to_owned(self) -> Self::Owned {
        *self
    }
}

impl<'a, P: Primitive, const SIZE: usize> ScalarMut<'a> for &'a mut [P; SIZE] {
    type Owned = [P; SIZE];

    #[inline]
    fn to_owned(self) -> Self::Owned {
        ToOwned::to_owned(self)
    }
}
