use {
    crate::{Arbitrary, Destructured, Result, Unstructured},
    core::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering},
};

/// Returns false, not an error, if this `Unstructured` [is empty][Unstructured::is_empty].
impl<'a> Arbitrary<'a> for AtomicBool {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&self.load(Ordering::SeqCst))
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <bool as Arbitrary<'a>>::size_hint(depth)
    }
}

/// Returns zero, not an error, if this `Unstructured` [is empty][Unstructured::is_empty].
impl<'a> Arbitrary<'a> for AtomicIsize {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&self.load(Ordering::SeqCst))
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <isize as Arbitrary<'a>>::size_hint(depth)
    }
}

/// Returns zero, not an error, if this `Unstructured` [is empty][Unstructured::is_empty].
impl<'a> Arbitrary<'a> for AtomicUsize {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Arbitrary::arbitrary(u).map(Self::new)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&self.load(Ordering::SeqCst))
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <usize as Arbitrary<'a>>::size_hint(depth)
    }
}
