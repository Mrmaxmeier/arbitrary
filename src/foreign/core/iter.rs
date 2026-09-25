use {
    crate::{Arbitrary, Destructured, Result, Unstructured},
    core::iter::{empty, Empty},
};

impl<'a, A> Arbitrary<'a> for Empty<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(_: &mut Unstructured<'a>) -> Result<Self> {
        Ok(empty())
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        let _ = d;
        Ok(())
    }

    #[inline]
    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
