use std::{
    mem::{size_of, transmute, ManuallyDrop},
    ptr::drop_in_place,
};

use thiserror::Error;

use super::{Label, LabelType};

pub type LabelValue = Label<Vec<u8>, [u8; 4], [u8; 16], i64, bool>;

#[derive(Debug)]
#[repr(transparent)]
pub struct AnyValue([u8; size_of::<LabelValue>()]);

impl From<LabelValue> for AnyValue {
    #[inline]
    fn from(value: LabelValue) -> Self {
        unsafe { transmute(ManuallyDrop::new(value)) }
    }
}

impl Clone for AnyValue {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { transmute(transmute::<_, &LabelValue>(&self.0).clone()) }
    }
}

impl AnyValue {
    #[inline]
    pub unsafe fn cast<T>(&self) -> &T {
        transmute(
            transmute::<_, *const u8>(self as *const _).offset(size_of::<usize>() as isize)
                as *const T,
        )
    }
}

impl Drop for AnyValue {
    #[inline]
    fn drop(&mut self) {
        unsafe { drop_in_place(transmute::<_, *mut LabelValue>(self)) };
    }
}

#[derive(Error, Debug)]
#[error("mismatch type of label value, expect {}, found: {}", .expect, .found)]
pub struct LabelTypeMismatch {
    pub expect: LabelType,
    pub found: LabelType,
}

pub trait TryAsRef<T: ?Sized> {
    type Error: std::error::Error;

    fn try_as_ref(&self) -> Result<&T, Self::Error>;
}

#[cfg(test)]
mod tests {
    use std::net::Ipv6Addr;

    use super::{AnyValue, LabelValue};

    #[test]
    fn cast_any_label_value() {
        let hello = Vec::from("hello");
        let a = LabelValue::String(hello.clone());
        let any = AnyValue::from(a);
        assert_eq!(&hello, unsafe { any.cast::<Vec<u8>>() });

        let a = LabelValue::IPv4([127, 0, 0, 1]);
        let any = AnyValue::from(a);
        assert_eq!(&[127, 0, 0, 1], unsafe { any.cast::<[u8; 4]>() });

        let ipv6: Ipv6Addr = "::1".parse().unwrap();
        let i = LabelValue::IPv6(ipv6.octets());
        let any = AnyValue::from(i);
        assert_eq!(&ipv6.octets(), unsafe { any.cast::<[u8; 16]>() });
    }
}
