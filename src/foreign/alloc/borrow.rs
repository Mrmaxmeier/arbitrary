use {
    crate::{size_hint, Arbitrary, Destructured, Result, Unstructured},
    std::borrow::{Cow, ToOwned},
};

impl<'a, A> Arbitrary<'a> for Cow<'a, A>
where
    A: ToOwned + ?Sized + 'a,
    <A as ToOwned>::Owned: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Cow::Owned)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        match self {
            Cow::Owned(owned) => d.push(owned),
            Cow::Borrowed(borrowed) => d.push(&(*borrowed).to_owned()),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        Self::try_size_hint(depth).unwrap_or_default()
    }

    #[inline]
    fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), crate::MaxRecursionReached> {
        size_hint::try_recursion_guard(depth, |depth| {
            <<A as ToOwned>::Owned as Arbitrary>::try_size_hint(depth)
        })
    }
}
