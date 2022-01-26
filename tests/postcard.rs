use postcard::*;
extern crate alloc;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
struct WrappedU8 {
    val: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct StringAndU8 {
    a: String,
    val: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct StringAndIsize {
    a: String,
    val: isize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TwoIsizes {
    a: isize,
    b: isize,
}

#[test]
fn test_postcard_u8() {
    let val = WrappedU8 { val: 1 };
    let bytes: Vec<u8> = to_allocvec(&val).unwrap();
    println!("value: {:?}", bytes);
    let result: WrappedU8 = from_bytes(&bytes).unwrap();
    dbg!(result);
}

#[test]
fn test_postcard_composite_w_u8() {
    let val = StringAndU8 {
        a: "hello".to_string(),
        val: 5,
    };
    let bytes: Vec<u8> = to_allocvec(&val).unwrap();
    println!("value: {:?}", bytes);
    let result: StringAndU8 = from_bytes(&bytes).unwrap();
    dbg!(result);
}

#[test]
fn test_postcard_composite_w_isize() {
    let val = StringAndIsize {
        a: "hello".to_string(),
        val: 0,
    };
    let bytes: Vec<u8> = to_allocvec(&val).unwrap();
    println!("value: {:?}", bytes);
    let result: StringAndIsize = from_bytes(&bytes).unwrap();
    dbg!(result);
}

#[test]
fn test_postcard_two_isizes() {
    let val = TwoIsizes { a: -1, b: 0 };
    let bytes: Vec<u8> = to_allocvec(&val).unwrap();
    println!("value: {:?}", bytes);
    let result: TwoIsizes = from_bytes(&bytes).unwrap();
    dbg!(result);
}

#[derive(Debug, Serialize, Deserialize)]
struct WrappedUsize {
    val: usize,
}

#[test]
fn test_postcard_usize() {
    let val = WrappedUsize { val: 0 };
    let bytes: Vec<u8> = to_allocvec(&val).unwrap();
    println!("value: {:?}", bytes);
    let result: WrappedUsize = from_bytes(&bytes).unwrap();
    dbg!(result);
}
