extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use std::collections::HashSet;
use std::iter::FromIterator;

use ohmers::{all, Ohmer};
use redis::Commands;
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Hash)]
struct Car {
    id: usize,
    name: String,
}
impl Ohmer for Car {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
    fn defaults() -> Self {
        Car {
            id: 0,
            name: "".to_string(),
        }
    }
}

#[test]
fn test_iter_all() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let _:bool = client.del("Car:all").unwrap();

    assert_eq!(
            all::<Car>(&client).unwrap().collect::<HashSet<_>>().len(),
            0
            );

    let mut toyota = Car::defaults();
    toyota.name = "Toyota".to_string();
    toyota.save(&client).unwrap();

    let mut mercedes = Car::defaults();
    mercedes.name = "Mercedes".to_string();
    mercedes.save(&client).unwrap();

    assert_eq!(
            all::<Car>(&client).unwrap().collect::<HashSet<_>>(),
            HashSet::from_iter(vec![toyota, mercedes])
            );
}
