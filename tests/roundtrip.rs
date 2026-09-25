#![cfg(feature = "derive")]
// Various structs/fields that we are deriving `Arbitrary` for aren't actually
// used except to exercise the derive.
#![allow(dead_code)]

//! Checks that `to_arbitrary_bytes` is the inverse of `arbitrary`: for random
//! inputs, decoding a value, encoding it again and decoding the result must
//! give back the same value.

use arbitrary::*;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::ffi::{CString, OsString};
use std::fmt::Debug;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{NonZeroI64, NonZeroU8, Wrapping};
use std::ops::{Bound, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn bytes(&mut self) -> Vec<u8> {
        let len = match self.next() % 4 {
            0 => self.next() % 8,
            1 => self.next() % 64,
            2 => self.next() % 600,
            _ => self.next() % 70_000,
        };
        (0..len)
            .map(|_| {
                // Bias towards small bytes so that continuation bools and
                // enum tags are not uniformly random.
                let x = self.next();
                if x % 3 == 0 {
                    (x >> 8) as u8 & 1
                } else {
                    (x >> 8) as u8
                }
            })
            .collect()
    }
}

const ITERATIONS: usize = 300;

/// Compares values by their `Debug` output, for types without (useful)
/// `PartialEq` impls.
fn check_by<T, K>(key: impl Fn(&T) -> K)
where
    T: for<'a> Arbitrary<'a> + Debug,
    K: PartialEq + Debug,
{
    let mut rng = Rng(0x1234_5678_9abc_def1 ^ std::any::type_name::<T>().len() as u64);
    for _ in 0..ITERATIONS {
        let input = rng.bytes();

        if let Ok(value) = T::arbitrary(&mut Unstructured::new(&input)) {
            let bytes =
                to_bytes(&value).unwrap_or_else(|e| panic!("failed to encode {:?}: {}", value, e));
            let decoded = T::arbitrary(&mut Unstructured::new(&bytes))
                .unwrap_or_else(|e| panic!("failed to decode {:?} from {:?}: {}", value, bytes, e));
            assert_eq!(key(&value), key(&decoded), "bytes: {:?}", bytes);
        }

        if let Ok(value) = T::arbitrary_take_rest(Unstructured::new(&input)) {
            let bytes = to_bytes_take_rest(&value)
                .unwrap_or_else(|e| panic!("failed to encode {:?}: {}", value, e));
            let decoded = T::arbitrary_take_rest(Unstructured::new(&bytes))
                .unwrap_or_else(|e| panic!("failed to decode {:?} from {:?}: {}", value, bytes, e));
            assert_eq!(key(&value), key(&decoded), "take_rest bytes: {:?}", bytes);
        }
    }
}

fn check<T>()
where
    T: for<'a> Arbitrary<'a> + Debug + PartialEq + Clone,
{
    check_by::<T, T>(T::clone);
}

fn check_debug<T>()
where
    T: for<'a> Arbitrary<'a> + Debug,
{
    check_by::<T, String>(|v| format!("{:?}", v));
}

/// Like `check`, but for types borrowing from the input.
macro_rules! check_borrowed {
    ($ty:ty) => {{
        let mut rng = Rng(0xfeed_f00d);
        for _ in 0..ITERATIONS {
            let input = rng.bytes();

            let value: $ty = Arbitrary::arbitrary(&mut Unstructured::new(&input)).unwrap();
            let bytes = to_bytes(&value).unwrap();
            let decoded: $ty = Arbitrary::arbitrary(&mut Unstructured::new(&bytes)).unwrap();
            assert_eq!(format!("{:?}", value), format!("{:?}", decoded));

            let value: $ty = Arbitrary::arbitrary_take_rest(Unstructured::new(&input)).unwrap();
            let bytes = to_bytes_take_rest(&value).unwrap();
            let decoded: $ty = Arbitrary::arbitrary_take_rest(Unstructured::new(&bytes)).unwrap();
            assert_eq!(format!("{:?}", value), format!("{:?}", decoded));
        }
    }};
}

macro_rules! roundtrip {
    ($( $name:ident: $check:expr; )*) => {
        $(
            #[test]
            fn $name() {
                $check;
            }
        )*
    };
}

roundtrip! {
    unit: check::<()>();
    bool_: check::<bool>();
    u8_: check::<u8>();
    u16_: check::<u16>();
    u32_: check::<u32>();
    u64_: check::<u64>();
    u128_: check::<u128>();
    usize_: check::<usize>();
    i8_: check::<i8>();
    i32_: check::<i32>();
    i128_: check::<i128>();
    isize_: check::<isize>();
    f32_: check_debug::<f32>();
    f64_: check_debug::<f64>();
    char_: check::<char>();
    nonzero: check::<(NonZeroU8, NonZeroI64)>();
    wrapping: check::<Wrapping<u16>>();
    reverse: check::<std::cmp::Reverse<u32>>();
    option: check::<Option<u32>>();
    result: check::<std::result::Result<u8, String>>();
    tuples: check::<(u8, bool, (String, Vec<u8>), char)>();
    tuple_string_last: check::<(u8, String)>();
    arrays: check::<[String; 3]>();
    empty_array: check::<[String; 0]>();
    array_of_vecs: check::<[Vec<u16>; 2]>();
    string: check::<String>();
    strings: check::<Vec<String>>();
    nested_strings: check::<(Vec<(String, Option<String>)>, String)>();
    vec: check::<Vec<u32>>();
    vec_deque: check::<VecDeque<u8>>();
    linked_list: check::<LinkedList<i16>>();
    btree_set: check::<BTreeSet<u8>>();
    btree_map: check::<BTreeMap<u8, String>>();
    hash_set: check::<HashSet<u16>>();
    hash_map: check::<HashMap<u8, Vec<u8>>>();
    binary_heap: check_by::<BinaryHeap<u8>, Vec<u8>>(|h| h.clone().into_sorted_vec());
    boxed: check::<(Box<u8>, Box<[u16]>, Box<str>)>();
    rc: check::<(Rc<u8>, Rc<[u16]>, Rc<str>)>();
    arc: check::<(Arc<u8>, Arc<[u16]>, Arc<str>)>();
    cow: check_borrowed!(Cow<'_, str>);
    cstring: check::<CString>();
    os_string: check::<OsString>();
    path_buf: check::<PathBuf>();
    duration: check::<Duration>();
    ranges: check::<(Range<u8>, RangeFrom<u8>, RangeInclusive<i8>, RangeTo<u8>, RangeToInclusive<u8>)>();
    bounds: check::<(Bound<u8>, Bound<u8>, Bound<u8>)>();
    ip: check::<(Ipv4Addr, Ipv6Addr, IpAddr)>();
    socket: check::<(SocketAddrV4, SocketAddrV6, SocketAddr)>();
    ref_cell: check::<RefCell<u32>>();
    mutex: check_by::<Mutex<u32>, u32>(|m| *m.lock().unwrap());
    atomics: check_by::<(std::sync::atomic::AtomicBool, std::sync::atomic::AtomicUsize), String>(|v| format!("{:?}", v));
    phantom: check::<std::marker::PhantomData<String>>();
    derived_struct: check::<Struct>();
    derived_tuple_struct: check::<TupleStruct>();
    derived_enum: check::<Enum>();
    derived_recursive: check::<Recursive>();
    derived_generic: check::<Generic<String, u8>>();
    derived_skip: check::<WithSkipped>();
    derived_default: check::<WithDefault>();
    derived_lifetime: check_borrowed!(Borrowed<'_>);
    borrowed_str: check_borrowed!(&'_ str);
    borrowed_slice: check_borrowed!(&'_ [u8]);
    borrowed_tuple: check_borrowed!((&'_ str, &'_ [u8]));
}

#[derive(Arbitrary, Debug, Clone, PartialEq)]
struct Struct {
    a: u8,
    b: String,
    c: Vec<(bool, char)>,
    r#fn: Option<u64>,
}

#[derive(Arbitrary, Debug, Clone, PartialEq)]
struct TupleStruct(u16, String, Vec<u8>);

#[derive(Arbitrary, Debug, Clone, PartialEq)]
enum Enum {
    Unit,
    Tuple(u8, String),
    Struct { x: i32, y: Vec<String> },
    Other,
    AndAnother(bool),
}

#[derive(Arbitrary, Debug, Clone, PartialEq)]
enum Recursive {
    Leaf(u8),
    Node(Vec<Recursive>, String),
    Boxed(Box<Recursive>),
}

#[derive(Arbitrary, Debug, Clone, PartialEq)]
struct Generic<T, U> {
    t: T,
    u: Vec<U>,
}

#[derive(Arbitrary, Debug, Clone, PartialEq)]
enum WithSkipped {
    A(u8),
    #[arbitrary(skip)]
    Skipped(String),
    B(String),
}

#[derive(Arbitrary, Debug, Clone, PartialEq)]
struct WithDefault {
    a: u8,
    #[arbitrary(default)]
    b: String,
    #[arbitrary(value = 42)]
    c: u32,
    d: String,
    #[arbitrary(default)]
    e: u8,
}

#[derive(Arbitrary, Debug)]
struct Borrowed<'a> {
    s: &'a str,
    b: &'a [u8],
    v: Vec<&'a str>,
}

#[test]
fn unencodable() {
    #[allow(clippy::reversed_empty_ranges)]
    let range = 5u8..3;
    assert_eq!(to_bytes(&range), Err(Error::Unencodable));
    assert_eq!(
        to_bytes(&WithSkipped::Skipped("x".into())),
        Err(Error::Unencodable)
    );
    assert_eq!(
        to_bytes(&std::cell::Cell::new(1u8)),
        Err(Error::Unencodable)
    );

    #[derive(Arbitrary, Debug)]
    struct WithFn {
        #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=100))]
        x: u8,
    }
    assert_eq!(to_bytes(&WithFn { x: 1 }), Err(Error::Unencodable));

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let not_utf8 = OsString::from_vec(vec![0xff]);
        assert_eq!(to_bytes(&not_utf8), Err(Error::Unencodable));
    }
}

#[test]
fn encodes_arbitrary_values() {
    // Values that `arbitrary` is unlikely to produce from random bytes.
    let value = Struct {
        a: 7,
        b: "x".repeat(70_000),
        c: vec![(true, '🦀'); 300],
        r#fn: Some(u64::MAX),
    };
    let bytes = to_bytes(&value).unwrap();
    assert_eq!(
        Struct::arbitrary(&mut Unstructured::new(&bytes)).unwrap(),
        value
    );
    let bytes = to_bytes_take_rest(&value).unwrap();
    assert_eq!(
        Struct::arbitrary_take_rest(Unstructured::new(&bytes)).unwrap(),
        value
    );

    let value = Recursive::Node(
        vec![
            Recursive::Boxed(Box::new(Recursive::Leaf(3))),
            Recursive::Node(vec![], "".into()),
        ],
        "hi".into(),
    );
    let bytes = to_bytes(&value).unwrap();
    assert_eq!(
        Recursive::arbitrary(&mut Unstructured::new(&bytes)).unwrap(),
        value
    );
}

#[test]
fn known_encoding() {
    // Pin down the byte format, so that it doesn't silently diverge from
    // what `arbitrary` decodes.
    assert_eq!(to_bytes(&0x0403_0201u32).unwrap(), [1, 2, 3, 4]);
    assert_eq!(to_bytes(&vec![1u8, 2]).unwrap(), [1, 1, 1, 2, 0]);
    assert_eq!(to_bytes(&Some(true)).unwrap(), [1, 1]);
    // The length is read from the end.
    assert_eq!(to_bytes(&"ab").unwrap(), [b'a', b'b', 2]);
    assert_eq!(to_bytes_take_rest(&"ab").unwrap(), [b'a', b'b']);
    assert_eq!(to_bytes(&("ab", "c")).unwrap(), [b'a', b'b', b'c', 1, 2]);
    // Variant 3 of 5 needs a tag `t` with `(t * 5) >> 32 == 1`.
    assert_eq!(
        to_bytes(&Enum::Other).unwrap(),
        ((3u64 << 32).div_ceil(5) as u32).to_le_bytes()
    );
}
