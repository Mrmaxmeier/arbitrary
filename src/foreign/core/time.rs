use {
    crate::{size_hint, Arbitrary, Destructured, Result, Unstructured},
    core::time::Duration,
};

/// Returns zero, not an error, if this `Unstructured` [is empty][Unstructured::is_empty].
impl<'a> Arbitrary<'a> for Duration {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Self::new(
            <u64 as Arbitrary>::arbitrary(u)?,
            u.int_in_range(0..=999_999_999)?,
        ))
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&self.as_secs())?;
        d.push_int_in_range(0..=999_999_999, self.subsec_nanos())
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        size_hint::and(
            <u64 as Arbitrary>::size_hint(depth),
            <u32 as Arbitrary>::size_hint(depth),
        )
    }
}
