extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use std::collections::HashSet;
use std::iter::FromIterator;

use ohmers::{Ohmer};
use redis::Commands;
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Person {
    id: usize,
    name: String,
}

impl Ohmer for Person {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
    fn defaults() -> Self {
        Person {
            id: 0,
            name: "".to_string(),
        }
    }
    fn unique_fields<'a>(&self) -> HashSet<&'a str> { HashSet::from_iter(vec!["name"]) }
}

#[test]
fn test_find() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let oldlen:usize = client.hlen("Person:uniques:name").unwrap();
    let mut person = Person { id: 0, name: "Jane".to_string() };
    person.save(&client).unwrap();
    let len:usize = client.hlen("Person:uniques:name").unwrap();
    assert_eq!(oldlen + 1, len);
}
