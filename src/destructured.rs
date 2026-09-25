// Copyright © 2019 The Rust Fuzz Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Turning structured values back into raw bytes.
//!
//! [`Destructured`] is the inverse of [`Unstructured`]: where `Unstructured`
//! hands out values built from raw bytes, `Destructured` collects the raw
//! bytes that `Unstructured` would need to hand out a given sequence of
//! values.
//!
//! [`Unstructured`]: crate::Unstructured

use crate::unstructured::Int;
use crate::{Arbitrary, Error, Result};
use std::{mem, ops};

/// Encode `value` into bytes that [`Arbitrary::arbitrary`] decodes back into
/// `value`.
///
/// ```
/// use arbitrary::{Arbitrary, Unstructured};
///
/// let value: (u32, String, Vec<bool>) = (42, "hello".into(), vec![true, false]);
/// let bytes = arbitrary::to_bytes(&value).unwrap();
///
/// let mut u = Unstructured::new(&bytes);
/// assert_eq!(<(u32, String, Vec<bool>)>::arbitrary(&mut u).unwrap(), value);
/// ```
pub fn to_bytes<'a, T: Arbitrary<'a>>(value: &T) -> Result<Vec<u8>> {
    let mut d = Destructured::new();
    value.to_arbitrary_bytes(&mut d)?;
    d.finish()
}

/// Encode `value` into bytes that [`Arbitrary::arbitrary_take_rest`] decodes
/// back into `value`.
///
/// This is what you want for building seed inputs for fuzz targets that take
/// a typed input, such as `libfuzzer_sys::fuzz_target!(|input: MyType| ...)`.
///
/// ```
/// use arbitrary::{Arbitrary, Unstructured};
///
/// let value: (u32, String) = (42, "hello".into());
/// let bytes = arbitrary::to_bytes_take_rest(&value).unwrap();
///
/// let u = Unstructured::new(&bytes);
/// assert_eq!(<(u32, String)>::arbitrary_take_rest(u).unwrap(), value);
/// ```
pub fn to_bytes_take_rest<'a, T: Arbitrary<'a>>(value: &T) -> Result<Vec<u8>> {
    let mut d = Destructured::new();
    value.to_arbitrary_take_rest_bytes(&mut d)?;
    d.finish()
}

/// A builder for raw bytes that an [`Unstructured`] will turn back into a
/// given sequence of values.
///
/// Every `Unstructured` method that consumes data has a counterpart here, and
/// implementations of [`Arbitrary::to_arbitrary_bytes`] should call them in
/// the same order that the matching [`Arbitrary::arbitrary`] implementation
/// calls the `Unstructured` methods.
///
/// Byte sizes (see [`Unstructured::arbitrary_len`]) are read from the *end* of
/// the data, and how they are encoded depends on how much data remains when
/// they are read. They are therefore only written out in
/// [`Destructured::finish`], once all the data is known.
///
/// [`Unstructured`]: crate::Unstructured
/// [`Unstructured::arbitrary_len`]: crate::Unstructured::arbitrary_len
#[derive(Debug, Default, Clone)]
pub struct Destructured {
    data: Vec<u8>,
    // `(data.len() at the time of the push, byte size)` for every byte size,
    // in the order that `Unstructured` will read them.
    byte_sizes: Vec<(usize, usize)>,
}

impl Destructured {
    /// Create a new, empty `Destructured`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append the encoding of `value`, i.e. the inverse of
    /// [`Unstructured::arbitrary`](crate::Unstructured::arbitrary).
    pub fn push<'a, T: Arbitrary<'a>>(&mut self, value: &T) -> Result<()> {
        value.to_arbitrary_bytes(self)
    }

    /// Append raw bytes, i.e. the inverse of
    /// [`Unstructured::bytes`](crate::Unstructured::bytes) and
    /// [`Unstructured::fill_buffer`](crate::Unstructured::fill_buffer).
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// The inverse of
    /// [`Unstructured::arbitrary_len::<ElementType>`](crate::Unstructured::arbitrary_len).
    ///
    /// `Unstructured` only produces lengths whose byte size (`len` times the
    /// element size derived from `ElementType::size_hint`) does not exceed
    /// the amount of data remaining after the length itself. If the data
    /// pushed after this call is too short for that, [`Destructured::finish`]
    /// returns [`Error::Unencodable`].
    pub fn push_arbitrary_len<'a, ElementType: Arbitrary<'a>>(&mut self, len: usize) -> Result<()> {
        let (lower, upper) = <ElementType as Arbitrary>::size_hint(0);
        let elem_size = upper.unwrap_or(lower * 2);
        let elem_size = std::cmp::max(1, elem_size);
        let byte_size = len.checked_mul(elem_size).ok_or(Error::Unencodable)?;
        self.byte_sizes.push((self.data.len(), byte_size));
        Ok(())
    }

    /// The inverse of
    /// [`Unstructured::int_in_range`](crate::Unstructured::int_in_range).
    pub fn push_int_in_range<T: Int>(
        &mut self,
        range: ops::RangeInclusive<T>,
        value: T,
    ) -> Result<()> {
        let (start, end) = (*range.start(), *range.end());
        assert!(
            start <= end,
            "`arbitrary::Destructured::push_int_in_range` requires a non-empty range"
        );
        if value < start || value > end {
            return Err(Error::Unencodable);
        }
        let start = start.to_unsigned();
        let delta = end.to_unsigned().wrapping_sub(start).to_u128();
        let offset = value.to_unsigned().wrapping_sub(start).to_u128();
        let mut buf = [0; 16];
        let n = int_in_range_bytes(delta, offset, mem::size_of::<T>(), &mut buf);
        self.push_bytes(&buf[..n]);
        Ok(())
    }

    /// The inverse of
    /// [`Unstructured::choose_index`](crate::Unstructured::choose_index).
    pub fn push_choose_index(&mut self, len: usize, index: usize) -> Result<()> {
        if index >= len {
            return Err(Error::Unencodable);
        }
        self.push_int_in_range(0..=len - 1, index)
    }

    /// The inverse of [`Unstructured::ratio`](crate::Unstructured::ratio).
    pub fn push_ratio<T: Int>(&mut self, numerator: T, denominator: T, value: bool) -> Result<()> {
        assert!(T::ZERO < numerator);
        assert!(numerator <= denominator);
        let x = if value { T::ONE } else { denominator };
        if (x <= numerator) != value {
            return Err(Error::Unencodable);
        }
        self.push_int_in_range(T::ONE..=denominator, x)
    }

    /// The inverse of
    /// [`Unstructured::arbitrary_iter`](crate::Unstructured::arbitrary_iter)
    /// and
    /// [`Unstructured::arbitrary_take_rest_iter`](crate::Unstructured::arbitrary_take_rest_iter).
    pub fn push_iter<'a, 'b, ElementType, I>(&mut self, elements: I) -> Result<()>
    where
        ElementType: Arbitrary<'a> + 'b,
        I: IntoIterator<Item = &'b ElementType>,
    {
        for element in elements {
            self.push(&true)?;
            self.push(element)?;
        }
        self.push(&false)
    }

    /// The number of bytes pushed so far, not counting byte sizes, which are
    /// only encoded in [`Destructured::finish`].
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether nothing has been pushed yet.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.byte_sizes.is_empty()
    }

    /// Finish encoding and return the raw bytes.
    pub fn finish(self) -> Result<Vec<u8>> {
        let Destructured {
            mut data,
            byte_sizes,
        } = self;

        // Encode the byte sizes in reverse reading order: when a byte size is
        // read, all the data after its position and all byte sizes that are
        // read later are still remaining, and the amount of remaining data
        // determines how the byte size is encoded.
        let mut encoded: Vec<([u8; 8], usize)> = Vec::with_capacity(byte_sizes.len());
        let mut later_sizes_len = 0usize;
        for &(position, byte_size) in byte_sizes.iter().rev() {
            // The amount of data that `Unstructured::arbitrary_byte_size` will
            // see, minus the bytes used to encode the byte size itself.
            let rest = (data.len() - position) + later_sizes_len;
            if byte_size > rest {
                return Err(Error::Unencodable);
            }
            let bytes = if rest as u64 <= u8::MAX as u64 {
                1
            } else if rest as u64 <= u16::MAX as u64 {
                2
            } else if rest as u64 <= u32::MAX as u64 {
                4
            } else {
                8
            };
            let mut buf = [0; 16];
            int_in_range_bytes(rest as u128, byte_size as u128, bytes, &mut buf);
            let mut chunk = [0; 8];
            chunk.copy_from_slice(&buf[..8]);
            encoded.push((chunk, bytes));
            later_sizes_len += bytes;
        }

        // The first byte size to be read is at the very end.
        for (chunk, bytes) in encoded {
            data.extend_from_slice(&chunk[..bytes]);
        }
        Ok(data)
    }
}

/// Write the bytes `Unstructured::int_in_range_impl` needs to produce
/// `start + offset` from a range `start..=start + delta` over an integer of
/// `int_size` bytes. Returns the number of bytes it will consume.
fn int_in_range_bytes(delta: u128, offset: u128, int_size: usize, buf: &mut [u8; 16]) -> usize {
    debug_assert!(offset <= delta);
    let mut n = 0;
    while n < int_size && (delta >> (n * 8)) > 0 {
        n += 1;
    }
    // Consumed bytes are combined most significant first. Since
    // `offset <= delta < 256^n`, the final modulo is a no-op.
    for (i, byte) in buf[..n].iter_mut().enumerate() {
        *byte = (offset >> ((n - 1 - i) * 8)) as u8;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Unstructured;

    #[test]
    fn int_in_range_roundtrip() {
        fn check<T: Int + std::fmt::Debug>(range: ops::RangeInclusive<T>, value: T) {
            let mut d = Destructured::new();
            d.push_int_in_range(range.clone(), value).unwrap();
            let bytes = d.finish().unwrap();
            let mut u = Unstructured::new(&bytes);
            assert_eq!(u.int_in_range(range).unwrap(), value);
            assert!(u.is_empty());
        }
        check(0u8..=0, 0);
        check(0u8..=255, 200);
        check(3u8..=7, 5);
        check(-128i8..=127, -3);
        check(-5i32..=1_000_000, -5);
        check(-5i32..=1_000_000, 999_999);
        check(0u64..=u64::MAX, u64::MAX - 1);
        check(i128::MIN..=i128::MAX, -12345);
        check(10u32..=u32::MAX, 0xdead_beef);
    }

    #[test]
    fn byte_size_roundtrip() {
        for &(len, trailing) in &[
            (0, 0),
            (0, 1),
            (1, 0),
            (5, 100),
            (255, 0),
            (256, 0),
            (100, 200),
            (70_000, 3),
        ] {
            let payload: Vec<u8> = (0..len).map(|i| i as u8).collect();
            let tail = vec![0xAA; trailing];
            let mut d = Destructured::new();
            d.push_arbitrary_len::<u8>(len).unwrap();
            d.push_bytes(&payload);
            d.push_bytes(&tail);
            let bytes = d.finish().unwrap();

            let mut u = Unstructured::new(&bytes);
            assert_eq!(u.arbitrary_len::<u8>().unwrap(), len);
            assert_eq!(u.bytes(len).unwrap(), &payload[..]);
            assert_eq!(u.take_rest(), &tail[..]);
        }
    }

    #[test]
    fn byte_size_too_large() {
        let mut d = Destructured::new();
        d.push_arbitrary_len::<u8>(3).unwrap();
        d.push_bytes(&[1, 2]);
        assert_eq!(d.finish(), Err(Error::Unencodable));
    }

    #[test]
    fn ratio_roundtrip() {
        for &(num, den) in &[(1u8, 1u8), (1, 2), (3, 7), (7, 7)] {
            for &value in &[true, false] {
                let mut d = Destructured::new();
                match d.push_ratio(num, den, value) {
                    Ok(()) => {
                        let bytes = d.finish().unwrap();
                        let mut u = Unstructured::new(&bytes);
                        assert_eq!(u.ratio(num, den).unwrap(), value);
                    }
                    Err(e) => {
                        assert_eq!(e, Error::Unencodable);
                        assert!(!value && num == den);
                    }
                }
            }
        }
    }
}
