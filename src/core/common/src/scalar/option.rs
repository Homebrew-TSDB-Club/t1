use super::{Scalar, ScalarMut, ScalarRef};

impl<S: Scalar> Scalar for Option<S> {
    type Ref<'a> = Option<S::Ref<'a>>;
    type Mut<'a> = Option<S::Mut<'a>>;

    #[inline]
    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_ref().map(|s| s.as_ref())
    }

    #[inline]
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self.as_mut().map(|s| s.as_mut())
    }
}

impl<'a, S: ScalarRef<'a>> ScalarRef<'a> for Option<S> {
    type Owned = Option<S::Owned>;

    #[inline]
    fn to_owned(self) -> Self::Owned {
        self.map(ScalarRef::to_owned)
    }
}

impl<'a, S: ScalarMut<'a>> ScalarMut<'a> for Option<S> {
    type Owned = Option<S::Owned>;

    #[inline]
    fn to_owned(self) -> Self::Owned {
        self.map(|this| this.to_owned())
    }
}
