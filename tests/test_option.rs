extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{get, Ohmer};
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Person {
    id: usize,
    name: String,
    father_name: Option<String>,
}

impl Default for Person {
    fn default() -> Self {
        Person {
            id: 0,
            name: "".to_string(),
            father_name: None,
        }
    }
}
impl Ohmer for Person {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[test]
fn test_option_some() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut person = Person::default();
    person.name = "Alice".to_string();
    person.father_name = Some("Bob".to_string());
    person.save(&client).unwrap();

    let person2 = get(person.id, &client).unwrap();
    assert_eq!(person, person2);
}

#[test]
fn test_option_none() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut person = Person::default();
    person.name = "Alice".to_string();
    person.save(&client).unwrap();

    let person2 = get(person.id, &client).unwrap();
    assert_eq!(person, person2);
}
