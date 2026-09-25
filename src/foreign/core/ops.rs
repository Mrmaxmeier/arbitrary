use {
    crate::{size_hint, Arbitrary, Destructured, Error, MaxRecursionReached, Result, Unstructured},
    core::{
        mem,
        ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
    },
};

macro_rules! impl_range {
    (
        $range:ty,
        $value_closure:expr,
        $value_ty:ty,
        $fun:ident($fun_closure:expr),
        $size_hint_closure:expr,
        $encodable:ident
    ) => {
        impl<'a, A> Arbitrary<'a> for $range
        where
            A: Arbitrary<'a> + Clone + PartialOrd,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
                let value: $value_ty = Arbitrary::arbitrary(u)?;
                Ok($fun(value, $fun_closure))
            }

            fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
                #[allow(clippy::redundant_closure_call)]
                let value: $value_ty = ($value_closure)(self);
                if !$encodable(&value) {
                    return Err(Error::Unencodable);
                }
                d.push(&value)
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                Self::try_size_hint(depth).unwrap_or_default()
            }

            #[inline]
            fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
                #[allow(clippy::redundant_closure_call)]
                $size_hint_closure(depth)
            }
        }
    };
}
impl_range!(
    Range<A>,
    |r: &Range<A>| (r.start.clone(), r.end.clone()),
    (A, A),
    bounded_range(|(a, b)| a..b),
    |depth| Ok(crate::size_hint::and(
        <A as Arbitrary>::try_size_hint(depth)?,
        <A as Arbitrary>::try_size_hint(depth)?,
    )),
    bounded_range_encodable
);
impl_range!(
    RangeFrom<A>,
    |r: &RangeFrom<A>| r.start.clone(),
    A,
    unbounded_range(|a| a..),
    |depth| <A as Arbitrary>::try_size_hint(depth),
    unbounded_range_encodable
);
impl_range!(
    RangeInclusive<A>,
    |r: &RangeInclusive<A>| (r.start().clone(), r.end().clone()),
    (A, A),
    bounded_range(|(a, b)| a..=b),
    |depth| Ok(crate::size_hint::and(
        <A as Arbitrary>::try_size_hint(depth)?,
        <A as Arbitrary>::try_size_hint(depth)?,
    )),
    bounded_range_encodable
);
impl_range!(
    RangeTo<A>,
    |r: &RangeTo<A>| r.end.clone(),
    A,
    unbounded_range(|b| ..b),
    |depth| <A as Arbitrary>::try_size_hint(depth),
    unbounded_range_encodable
);
impl_range!(
    RangeToInclusive<A>,
    |r: &RangeToInclusive<A>| r.end.clone(),
    A,
    unbounded_range(|b| ..=b),
    |depth| <A as Arbitrary>::try_size_hint(depth),
    unbounded_range_encodable
);

pub(crate) fn bounded_range<CB, I, R>(bounds: (I, I), cb: CB) -> R
where
    CB: Fn((I, I)) -> R,
    I: PartialOrd,
    R: RangeBounds<I>,
{
    let (mut start, mut end) = bounds;
    if start > end {
        mem::swap(&mut start, &mut end);
    }
    cb((start, end))
}

// `bounded_range` never produces a range with `start > end`.
fn bounded_range_encodable<I: PartialOrd>((start, end): &(I, I)) -> bool {
    start.partial_cmp(end) != Some(core::cmp::Ordering::Greater)
}

fn unbounded_range_encodable<I>(_: &I) -> bool {
    true
}

pub(crate) fn unbounded_range<CB, I, R>(bound: I, cb: CB) -> R
where
    CB: Fn(I) -> R,
    R: RangeBounds<I>,
{
    cb(bound)
}

impl<'a, A> Arbitrary<'a> for Bound<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        match u.int_in_range::<u8>(0..=2)? {
            0 => Ok(Bound::Included(A::arbitrary(u)?)),
            1 => Ok(Bound::Excluded(A::arbitrary(u)?)),
            2 => Ok(Bound::Unbounded),
            _ => unreachable!(),
        }
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        match self {
            Bound::Included(value) => {
                d.push_int_in_range::<u8>(0..=2, 0)?;
                d.push(value)
            }
            Bound::Excluded(value) => {
                d.push_int_in_range::<u8>(0..=2, 1)?;
                d.push(value)
            }
            Bound::Unbounded => d.push_int_in_range::<u8>(0..=2, 2),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        Ok(size_hint::or(
            size_hint::and((1, Some(1)), A::try_size_hint(depth)?),
            (1, Some(1)),
        ))
    }
}
