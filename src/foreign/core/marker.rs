use {
    crate::{Arbitrary, Destructured, Result, Unstructured},
    core::marker::{PhantomData, PhantomPinned},
};

impl<'a, A> Arbitrary<'a> for PhantomData<A>
where
    A: ?Sized,
{
    fn arbitrary(_: &mut Unstructured<'a>) -> Result<Self> {
        Ok(PhantomData)
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

impl<'a> Arbitrary<'a> for PhantomPinned {
    fn arbitrary(_: &mut Unstructured<'a>) -> Result<Self> {
        Ok(PhantomPinned)
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
