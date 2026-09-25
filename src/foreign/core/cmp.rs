use {
    crate::{size_hint, Arbitrary, Destructured, Result, Unstructured},
    core::cmp::Reverse,
};

impl<'a, A> Arbitrary<'a> for Reverse<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&self.0)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), crate::MaxRecursionReached> {
        size_hint::try_recursion_guard(depth, <A as Arbitrary>::try_size_hint)
    }
}
