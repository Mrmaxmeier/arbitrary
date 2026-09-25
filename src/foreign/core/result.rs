use crate::{size_hint, Arbitrary, Destructured, Error, MaxRecursionReached, Unstructured};

impl<'a, T, E> Arbitrary<'a> for Result<T, E>
where
    T: Arbitrary<'a>,
    E: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        Ok(if <bool as Arbitrary<'a>>::arbitrary(u)? {
            Ok(<T as Arbitrary>::arbitrary(u)?)
        } else {
            Err(<E as Arbitrary>::arbitrary(u)?)
        })
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<(), Error> {
        match self {
            Ok(value) => {
                d.push(&true)?;
                d.push(value)
            }
            Err(value) => {
                d.push(&false)?;
                d.push(value)
            }
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        Ok(size_hint::and(
            <bool as Arbitrary>::size_hint(depth),
            size_hint::or(
                <T as Arbitrary>::try_size_hint(depth)?,
                <E as Arbitrary>::try_size_hint(depth)?,
            ),
        ))
    }
}
