use {
    crate::{Arbitrary, Destructured, MaxRecursionReached, Result, Unstructured},
    std::sync::Mutex,
};

impl<'a, A> Arbitrary<'a> for Mutex<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        match self.lock() {
            Ok(guard) => d.push(&*guard),
            Err(poisoned) => d.push(&*poisoned.into_inner()),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
        A::try_size_hint(depth)
    }
}
