use crate::{Arbitrary, Destructured, Result, Unstructured};

impl<'a> Arbitrary<'a> for &'a [u8] {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let len = u.arbitrary_len::<u8>()?;
        u.bytes(len)
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
        Ok(u.take_rest())
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push_arbitrary_len::<u8>(self.len())?;
        d.push_bytes(self);
        Ok(())
    }

    fn to_arbitrary_take_rest_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push_bytes(self);
        Ok(())
    }

    #[inline]
    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (0, None)
    }
}
