use super::bitmap::{Bitmap, BitmapRef, BitmapRefMut};
use crate::primitive::Primitive;

pub enum MaybeRef<'r, S: Scalar> {
    Owned(S),
    Ref(S::Ref<'r>),
}

pub trait Scalar: 'static + Clone + Sized {
    type Ref<'a>: ScalarRef<'a>
    where
        Self: 'a;
    type RefMut<'a>: ScalarRefMut<'a>
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Ref<'_>;
}

pub trait ScalarRef<'a>: Clone {
    type Owned: Scalar;

    fn to_owned(self) -> Self::Owned;
}

pub trait ScalarRefMut<'a> {
    type Owned: Scalar;
}

impl<P: Primitive> Scalar for P {
    type Ref<'a> = &'a P;
    type RefMut<'a> = &'a mut P;

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<P: Primitive> Scalar for Vec<P> {
    type Ref<'a> = &'a [P];
    type RefMut<'a> = &'a mut [P];

    fn as_ref(&self) -> Self::Ref<'_> {
        &self[..]
    }
}

impl<S: Scalar> Scalar for Option<S> {
    type Ref<'a> = Option<S::Ref<'a>>;
    type RefMut<'a> = Option<S::RefMut<'a>>;

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_ref().map(|s| s.as_ref())
    }
}

impl<'a, P: Primitive> ScalarRef<'a> for &'a P {
    type Owned = P;

    fn to_owned(self) -> Self::Owned {
        *self
    }
}

impl<'a, P: Primitive> ScalarRef<'a> for &'a [P] {
    type Owned = Vec<P>;

    fn to_owned(self) -> Self::Owned {
        Vec::from(self)
    }
}

impl<'a, S: ScalarRef<'a>> ScalarRef<'a> for Option<S> {
    type Owned = Option<S::Owned>;

    fn to_owned(self) -> Self::Owned {
        self.map(ScalarRef::to_owned)
    }
}

impl<'a, P: Primitive> ScalarRefMut<'a> for &'a mut P {
    type Owned = P;
}

impl<'a, P: Primitive> ScalarRefMut<'a> for &'a mut [P] {
    type Owned = Vec<P>;
}

impl<'a, S: ScalarRefMut<'a>> ScalarRefMut<'a> for Option<S> {
    type Owned = Option<S::Owned>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct NullableFixedSizedList<P: Primitive> {
    pub(crate) data: Vec<P>,
    pub(crate) validity: Bitmap,
}

impl<P: Primitive> NullableFixedSizedList<P> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            validity: Bitmap::new(),
        }
    }
}

impl<P: Primitive> From<Vec<Option<P>>> for NullableFixedSizedList<P> {
    fn from(raw: Vec<Option<P>>) -> Self {
        let mut validity = Bitmap::new();
        let mut data = Vec::new();
        for item in raw {
            if let Some(item) = item {
                validity.push(true);
                data.push(item.clone());
            } else {
                validity.push(false);
                data.push(Default::default());
            }
        }
        validity.align();
        Self { data, validity }
    }
}

#[derive(Debug, PartialEq)]
pub struct NullableFixedSizeListRef<'a, P: Primitive> {
    pub(crate) validity: BitmapRef<'a>,
    pub(crate) data: &'a [P],
}

impl<P: Primitive> Clone for NullableFixedSizeListRef<'_, P> {
    fn clone(&self) -> Self {
        Self {
            validity: self.validity.clone(),
            data: self.data,
        }
    }
}

impl<'a, P: Primitive> NullableFixedSizeListRef<'a, P> {
    pub(crate) fn new(validity: BitmapRef<'a>, data: &'a [P]) -> Self {
        Self { validity, data }
    }

    pub fn get(&self, n: usize) -> Option<Option<&P>> {
        if self.data.len() > n {
            if self.validity.get_bit(n) {
                Some(Some(&self.data[n]))
            } else {
                Some(None)
            }
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NullableFixedSizeListRefMut<'a, P: Primitive> {
    pub(crate) validity: BitmapRefMut<'a>,
    pub(crate) data: &'a mut [P],
}

impl<'a, P: Primitive> NullableFixedSizeListRefMut<'a, P> {
    pub(crate) fn new(validity: BitmapRefMut<'a>, data: &'a mut [P]) -> Self {
        Self { validity, data }
    }

    pub fn get(&self, n: usize) -> Option<Option<&P>> {
        if self.validity.len() > n {
            if self.validity.get_bit(n) {
                Some(Some(&self.data[n]))
            } else {
                Some(None)
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, n: usize, value: Option<P>) {
        self.validity.insert(n, value.is_some());
        if let Some(value) = value {
            self.data[n] = value;
        }
    }
}

impl<P: Primitive> Scalar for NullableFixedSizedList<P> {
    type Ref<'a> = NullableFixedSizeListRef<'a, P>;
    type RefMut<'a> = NullableFixedSizeListRefMut<'a, P>;

    fn as_ref(&self) -> Self::Ref<'_> {
        NullableFixedSizeListRef::new(self.validity.as_ref(), &self.data[..])
    }
}

impl<'a, P: Primitive> ScalarRef<'a> for NullableFixedSizeListRef<'a, P> {
    type Owned = NullableFixedSizedList<P>;

    fn to_owned(self) -> Self::Owned {
        NullableFixedSizedList {
            data: ToOwned::to_owned(self.data),
            validity: self.validity.to_owned(),
        }
    }
}

impl<'a, P: Primitive> ScalarRefMut<'a> for NullableFixedSizeListRefMut<'a, P> {
    type Owned = NullableFixedSizedList<P>;
}

impl<P: Primitive, const SIZE: usize> Scalar for [P; SIZE] {
    type Ref<'a> = &'a [P; SIZE]
    where
        Self: 'a;

    type RefMut<'a> = &'a mut [P; SIZE]
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<'a, P: Primitive, const SIZE: usize> ScalarRef<'a> for &'a [P; SIZE] {
    type Owned = [P; SIZE];

    fn to_owned(self) -> Self::Owned {
        *self
    }
}

impl<'a, P: Primitive, const SIZE: usize> ScalarRefMut<'a> for &'a mut [P; SIZE] {
    type Owned = [P; SIZE];
}

#[cfg(test)]
mod tests {
    use super::{NullableFixedSizedList, Scalar};

    #[test]
    fn test_list() {
        let list = NullableFixedSizedList::from(vec![None, Some(1)]);
        assert!(list.as_ref().get(0) == Some(None));
        assert!(list.as_ref().get(1).map(|s| s.cloned()) == Some(Some(1)));
        assert!(list.as_ref().get(2) == None);
    }
}
