use {
    crate::{Arbitrary, Destructured, Result, Unstructured},
    std::string::String,
};

impl<'a> Arbitrary<'a> for String {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        <&str as Arbitrary>::arbitrary(u).map(Into::into)
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
        <&str as Arbitrary>::arbitrary_take_rest(u).map(Into::into)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&self.as_str())
    }

    fn to_arbitrary_take_rest_bytes(&self, d: &mut Destructured) -> Result<()> {
        self.as_str().to_arbitrary_take_rest_bytes(d)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <&str as Arbitrary>::size_hint(depth)
    }
}
