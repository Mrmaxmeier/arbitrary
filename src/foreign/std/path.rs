use {
    crate::{Arbitrary, Destructured, Result, Unstructured},
    std::{ffi::OsString, path::PathBuf},
};

impl<'a> Arbitrary<'a> for PathBuf {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        <OsString as Arbitrary>::arbitrary(u).map(From::from)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        d.push(&self.as_os_str().to_owned())
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <OsString as Arbitrary>::size_hint(depth)
    }
}
