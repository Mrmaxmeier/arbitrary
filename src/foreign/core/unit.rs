use crate::{Arbitrary, Destructured, Result, Unstructured};

impl<'a> Arbitrary<'a> for () {
    fn arbitrary(_: &mut Unstructured<'a>) -> Result<Self> {
        Ok(())
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
