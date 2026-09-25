use crate::{size_hint, Arbitrary, Destructured, MaxRecursionReached, Result, Unstructured};

macro_rules! arbitrary_tuple {
    () => {};
    ($last: ident $($xs: ident)*) => {
        arbitrary_tuple!($($xs)*);

        impl<'a, $($xs,)* $last> Arbitrary<'a> for ($($xs,)* $last,)
        where
            $($xs: Arbitrary<'a>,)*
            $last: Arbitrary<'a>,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
                Ok(($($xs::arbitrary(u)?,)* Arbitrary::arbitrary(u)?,))
            }

            #[allow(unused_mut, non_snake_case)]
            fn arbitrary_take_rest(mut u: Unstructured<'a>) -> Result<Self> {
                $(let $xs = $xs::arbitrary(&mut u)?;)*
                let $last = $last::arbitrary_take_rest(u)?;
                Ok(($($xs,)* $last,))
            }

            #[allow(non_snake_case)]
            fn to_arbitrary_bytes(&self, d: &mut Destructured) -> Result<()> {
                let ($($xs,)* $last,) = self;
                $($xs.to_arbitrary_bytes(d)?;)*
                $last.to_arbitrary_bytes(d)
            }

            #[allow(non_snake_case)]
            fn to_arbitrary_take_rest_bytes(&self, d: &mut Destructured) -> Result<()> {
                let ($($xs,)* $last,) = self;
                $($xs.to_arbitrary_bytes(d)?;)*
                $last.to_arbitrary_take_rest_bytes(d)
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                Self::try_size_hint(depth).unwrap_or_default()
            }
            #[inline]
            fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), MaxRecursionReached> {
                Ok(size_hint::and_all(&[
                    <$last as Arbitrary>::try_size_hint(depth)?,
                    $( <$xs as Arbitrary>::try_size_hint(depth)?),*
                ]))
            }
        }
    };
}
arbitrary_tuple!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);
