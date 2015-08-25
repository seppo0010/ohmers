extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{get, Ohmer};
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Numbers {
    id: usize,
    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,
    usize: usize,
    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,
    isize: isize,
}

impl Default for Numbers {
    fn default() -> Self {
        Numbers {
            id: 0,
            u8: 0,
            u16: 0,
            u32: 0,
            u64: 0,
            usize: 0,
            i8: 0,
            i16: 0,
            i32: 0,
            i64: 0,
            isize: 0,
        }
    }
}

impl Ohmer for Numbers {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[test]
fn test_numbers_max() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut numbers = Numbers::default();
    numbers.u8 = std::u8::MAX;
    numbers.u16 = std::u16::MAX;
    numbers.u32 = std::u32::MAX;
    numbers.u64 = std::u64::MAX;
    numbers.usize = std::usize::MAX;
    numbers.i8 = std::i8::MAX;
    numbers.i16 = std::i16::MAX;
    numbers.i32 = std::i32::MAX;
    numbers.i64 = std::i64::MAX;
    numbers.isize = std::isize::MAX;
    numbers.save(&client).unwrap();

    let numbers2 = get(numbers.id, &client).unwrap();
    assert_eq!(numbers, numbers2);
}

#[test]
fn test_numbers_min() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut numbers = Numbers::default();
    numbers.u8 = std::u8::MIN;
    numbers.u16 = std::u16::MIN;
    numbers.u32 = std::u32::MIN;
    numbers.u64 = std::u64::MIN;
    numbers.usize = std::usize::MIN;
    numbers.i8 = std::i8::MIN;
    numbers.i16 = std::i16::MIN;
    numbers.i32 = std::i32::MIN;
    numbers.i64 = std::i64::MIN;
    numbers.isize = std::isize::MIN;
    numbers.save(&client).unwrap();

    let numbers2 = get(numbers.id, &client).unwrap();
    assert_eq!(numbers, numbers2);
}
