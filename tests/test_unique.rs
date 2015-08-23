extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use std::collections::HashSet;
use std::iter::FromIterator;

use ohmers::{Ohmer, OhmerError};
use redis::Commands;
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Thing {
    id: usize,
    name: String,
}

impl Ohmer for Thing {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
    fn defaults() -> Self {
        Thing {
            id: 0,
            name: "".to_string(),
        }
    }
    fn unique_fields<'a>(&self) -> HashSet<&'a str> { HashSet::from_iter(vec!["name"]) }
}

#[test]
fn test_find() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let _:() = client.del("Thing:uniques:name").unwrap();
    let mut person = Thing { id: 0, name: "Jane".to_string() };
    person.save(&client).unwrap();
    let len:usize = client.hlen("Thing:uniques:name").unwrap();
    assert_eq!(1, len);

    let mut person2 = Thing { id: 0, name: "Jane".to_string() };
    assert_eq!(person2.save(&client).unwrap_err(), OhmerError::UniqueIndexViolation("name".to_string()));
}
