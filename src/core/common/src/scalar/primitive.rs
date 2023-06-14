use super::{Scalar, ScalarMut, ScalarRef};
use crate::primitive::Primitive;

impl<P: Primitive> Scalar for P {
    type Ref<'a> = &'a P;
    type Mut<'a> = &'a mut P;

    #[inline]
    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }

    #[inline]
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}

impl<'a, P: Primitive> ScalarRef<'a> for &'a P {
    type Owned = P;

    #[inline]
    fn to_owned(self) -> Self::Owned {
        *self
    }
}

impl<'a, P: Primitive> ScalarMut<'a> for &'a mut P {
    type Owned = P;

    #[inline]
    fn to_owned(self) -> Self::Owned {
        *self
    }
}
