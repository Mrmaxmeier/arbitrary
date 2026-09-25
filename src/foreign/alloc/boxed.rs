use {
    crate::{size_hint, Arbitrary, Destructured, Result, Unstructured},
    std::boxed::Box,
};

impl<'a, A> Arbitrary<'a> for Box<A>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&**self)
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

impl<'a, A> Arbitrary<'a> for Box<[A]>
where
    A: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        u.arbitrary_iter()?.collect()
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push_iter(self.iter())
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
        u.arbitrary_take_rest_iter()?.collect()
    }

    #[inline]
    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<'a> Arbitrary<'a> for Box<str> {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        <String as Arbitrary>::arbitrary(u).map(|x| x.into_boxed_str())
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&&**self)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <String as Arbitrary>::size_hint(depth)
    }
}
