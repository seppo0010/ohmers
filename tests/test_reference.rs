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
impl Default for Person {
    fn default() -> Self {
        Person {
            id: 0,
            name: "".to_string(),
            father: Reference::new(),
            mother: Reference::new(),
        }
    }
}
impl Ohmer for Person {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[test]
fn test_reference() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let mut father = Person::default();
    father.name = "John".to_string();
    father.save(&client).unwrap();

    let mut mother = Person::default();
    mother.name = "Jane".to_string();
    mother.save(&client).unwrap();

    let mut person = Person::default();
    person.name = "Alice".to_string();
    person.father.set(&father);
    person.mother.set(&mother);
    person.save(&client).unwrap();

    let person2 = get(person.id, &client).unwrap();
    assert_eq!(person, person2);
    assert_eq!(person2.father.get(&client).unwrap(), father);
    assert_eq!(person2.mother.get(&client).unwrap(), mother);
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Location {
    id: usize,
    name: String,
}
impl Default for Location {
    fn default() -> Self {
        Location {
            id: 0,
            name: "".to_string(),
        }
    }
}
impl Ohmer for Location {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Event {
    id: usize,
    name: String,
    location: Reference<Location>
}
impl Default for Event {
    fn default() -> Self {
        Event {
            id: 0,
            name: "".to_string(),
            location: Reference::new(),
        }
    }
}
impl Ohmer for Event {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[test]
fn test_event_location() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let mut location = Location::default();
    location.name = "House".to_string();
    location.save(&client).unwrap();

    let mut event = Event::default();
    event.name = "Birthday Party".to_string();
    event.location.set(&location);
    assert_eq!(event.id, 0);
    event.save(&client).unwrap();
    assert!(event.id > 0);

    let event2:Event = get(event.id, &client).unwrap();
    assert_eq!(event2.name, "Birthday Party");
    assert_eq!(event2.location.get(&client).unwrap().name, "House");
}
