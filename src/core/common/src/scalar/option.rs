use super::{Scalar, ScalarMut, ScalarRef};

impl<S: Scalar> Scalar for Option<S> {
    type Ref<'a> = Option<S::Ref<'a>>;
    type RefMut<'a> = Option<S::RefMut<'a>>;

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_ref().map(|s| s.as_ref())
    }
}

impl<'a, S: ScalarRef<'a>> ScalarRef<'a> for Option<S> {
    type Owned = Option<S::Owned>;

    fn to_owned(self) -> Self::Owned {
        self.map(ScalarRef::to_owned)
    }
}

impl<'a, S: ScalarMut<'a>> ScalarMut<'a> for Option<S> {
    type Owned = Option<S::Owned>;
}
