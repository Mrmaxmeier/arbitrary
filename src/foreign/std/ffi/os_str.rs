use {
    crate::{Arbitrary, Destructured, Error, Result, Unstructured},
    std::ffi::OsString,
};

impl<'a> Arbitrary<'a> for OsString {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        <String as Arbitrary>::arbitrary(u).map(From::from)
    }

    fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
        // `arbitrary` only produces valid UTF-8.
        d.push(&self.to_str().ok_or(Error::Unencodable)?)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <String as Arbitrary>::size_hint(depth)
    }
}

// impl Arbitrary for Box<OsStr> {
//     fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
//         <OsString as Arbitrary>::arbitrary(u).map(|x| x.into_boxed_osstr())
//
//     }
// }
