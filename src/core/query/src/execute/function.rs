use anyhow::anyhow;
use chunk::mutable::column::field::FieldItemImpl;
use common::{
    scalar::{list::OptionalFixedList, Scalar},
    DynError,
};
use paste::paste;

pub type Function = fn(FieldItemImpl) -> Result<FieldItemImpl, DynError>;

#[allow(arithmetic_overflow)]
pub fn rate(item: FieldItemImpl) -> Result<FieldItemImpl, DynError> {
    macro_rules! rate {
        ($($label_type:ident), *) => {
            paste! {
            match item {
                $(
                FieldItemImpl::$label_type(item) => {
                    let shift_ref = item.as_ref();
                    let origin_ref = item.as_ref();
                    let mut shift = shift_ref.slice_from(1..).into_iter();
                    let mut origin = origin_ref.into_iter();
                    // TODO: with capacity
                    let mut rated = OptionalFixedList::new();
                    loop {
                        let (oi, si) = (origin.next(), shift.next());
                        if oi.is_none() || si.is_none() {
                            return Ok(FieldItemImpl::$label_type(rated));
                        }
                        let (oi, si) = unsafe { (oi.unwrap_unchecked(), si.unwrap_unchecked()) };
                        let item = match (oi, si) {
                            (Some(oi), Some(si)) => Some(*si - *oi),
                            _ => None,
                        };
                        rated.push(item);
                    }
                }
                )*
                FieldItemImpl::Bool(_) => Err(anyhow!("rate function does not support bool type").into()),
            }
            }
        };
    }
    rate!(UInt8, UInt16, UInt32, UInt64, Int8, Int16, Int32, Int64, Float32, Float64)
}

#[cfg(test)]
mod tests {
    use chunk::mutable::column::field::FieldItemImpl;
    use common::scalar::list::OptionalFixedList;

    use super::rate;

    #[test]
    fn test_rate() {
        let list = FieldItemImpl::Int32(OptionalFixedList::from(vec![
            Some(1),
            Some(3),
            Some(5),
            Some(7),
            Some(9),
            None,
            Some(12),
            Some(13),
        ]));
        let result = rate(list).unwrap();
        assert_eq!(
            result,
            FieldItemImpl::Int32(OptionalFixedList::from(vec![
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                None,
                None,
                Some(1)
            ]))
        );
    }
}
