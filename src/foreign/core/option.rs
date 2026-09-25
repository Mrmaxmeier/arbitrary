use crate::{size_hint, Arbitrary, Destructured, MaxRecursionReached, Result, Unstructured};

/// Returns `None`, not an error, if this `Unstructured` [is empty][Unstructured::is_empty].
impl<'a, A> Arbitrary<'a> for Option<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(if <bool as Arbitrary<'a>>::arbitrary(u)? {
            Some(Arbitrary::arbitrary(u)?)
        } else {
            None
        })
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        match self {
            Some(value) => {
                d.push(&true)?;
                d.push(value)
            }
            None => d.push(&false),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        Ok(size_hint::and(
            <bool as Arbitrary>::try_size_hint(depth)?,
            size_hint::or((0, Some(0)), <A as Arbitrary>::try_size_hint(depth)?),
        ))
    }
}
