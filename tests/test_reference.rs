extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{get, Ohmer, Reference};
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Person {
    id: usize,
    name: String,
    father: Reference<Person>,
    mother: Reference<Person>,
}
impl Ohmer for Person {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
    fn defaults() -> Self {
        Person {
            id: 0,
            name: "".to_string(),
            father: Reference::new(),
            mother: Reference::new(),
        }
    }
}

#[test]
fn test_reference() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let mut father = Person::defaults();
    father.name = "John".to_string();
    father.save(&client).unwrap();

    let mut mother = Person::defaults();
    mother.name = "Jane".to_string();
    mother.save(&client).unwrap();

    let mut person = Person::defaults();
    person.name = "Alice".to_string();
    person.father.set(&father);
    person.mother.set(&mother);
    person.save(&client).unwrap();

    let person2 = get(person.id, &client).unwrap();
    assert_eq!(person, person2);
    assert_eq!(person2.father.get(&client).unwrap(), father);
    assert_eq!(person2.mother.get(&client).unwrap(), mother);
}
