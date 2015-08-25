extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{get, Ohmer};
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Person {
    id: usize,
    name: String,
}
impl Default for Person {
    fn default() -> Self {
        Person {
            id: 0,
            name: "".to_string(),
        }
    }
}
impl Ohmer for Person {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[test]
fn test_save() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut person = Person { id: 0, name: "John".to_string() };
    person.save(&client).unwrap();
}

#[test]
fn test_save_load() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut person = Person { id: 0, name: "Jane".to_string() };
    person.save(&client).unwrap();
    let person2 = get(person.id, &client).unwrap();
    assert_eq!(person, person2);
}
