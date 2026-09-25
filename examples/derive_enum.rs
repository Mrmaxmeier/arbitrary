//! A simple example of deriving the `Arbitrary` trait for an `enum`.
//!
//! Note that this requires enabling the "derive" cargo feature.

// Various enums/fields that we are deriving `Arbitrary` for aren't actually
// used except to show off the derive.
#![allow(dead_code)]

use arbitrary::{Arbitrary, Unstructured};

#[derive(Arbitrary, Debug)]
enum MyEnum {
    Unit,
    Tuple(bool, u32),
    Struct {
        x: i8,
        y: (u8, i32),
    },

    #[arbitrary(skip)]
    Skipped(usize),
}

fn main() {
    let raw = b"This is some raw, unstructured data!";

    let mut unstructured = Unstructured::new(raw);

    let instance = MyEnum::arbitrary(&mut unstructured)
        .expect("`unstructured` has enough underlying data to create all variants of `MyEnum`");

    println!("Here is an arbitrary enum: {:?}", instance);

    // Values can also be turned back into raw data that recreates them.
    let raw = arbitrary::to_bytes(&instance).expect("`instance` is not a skipped variant");
    let instance_again = MyEnum::arbitrary(&mut Unstructured::new(&raw))
        .expect("`raw` has enough underlying data to recreate `instance`");

    println!("Here it is again, from {:?}: {:?}", raw, instance_again);
}
