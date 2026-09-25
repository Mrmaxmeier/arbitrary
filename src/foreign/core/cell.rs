use {
    crate::{Arbitrary, Destructured, Error, MaxRecursionReached, Result, Unstructured},
    core::cell::{Cell, RefCell, UnsafeCell},
};

impl<'a, A> Arbitrary<'a> for Cell<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        <A as Arbitrary<'a>>::try_size_hint(depth)
    }
}

impl<'a, A> Arbitrary<'a> for RefCell<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&*self.try_borrow().map_err(|_| Error::Unencodable)?)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        <A as Arbitrary<'a>>::try_size_hint(depth)
    }
}

impl<'a, A> Arbitrary<'a> for UnsafeCell<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        <A as Arbitrary<'a>>::try_size_hint(depth)
    }
}
