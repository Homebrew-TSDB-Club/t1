use super::{Scalar, ScalarMut, ScalarRef};
use crate::primitive::Primitive;

impl<P: Primitive> Scalar for Vec<P> {
    type Ref<'a> = &'a [P];
    type RefMut<'a> = &'a mut [P];

    fn as_ref(&self) -> Self::Ref<'_> {
        &self[..]
    }
}

impl<'a, P: Primitive> ScalarRef<'a> for &'a [P] {
    type Owned = Vec<P>;

    fn to_owned(self) -> Self::Owned {
        Vec::from(self)
    }
}

impl<'a, P: Primitive> ScalarMut<'a> for &'a mut [P] {
    type Owned = Vec<P>;
}
