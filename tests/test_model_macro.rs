#[macro_use(model)] extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use std::collections::HashSet;
use std::iter::FromIterator;

use ohmers::{get, Ohmer};
use redis::Commands;
use rustc_serialize::Encodable;

model!(Person {
        name:String = "".to_owned();
        age:u8 = 18;
        });

#[test]
fn test_model_macro() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let mut person = Person::default();
    assert_eq!(person.id, 0);
    assert_eq!(person.name, "".to_owned());
    assert_eq!(person.age, 18);
    assert_eq!(person.get_class_name(), "Person".to_owned());
    person.save(&client).unwrap();

    assert_eq!(get::<Person>(person.id, &client).unwrap(), person);
}

model!(UPerson {
        uniques { name:String = "".to_owned(); };
        age:u8 = 18;
        });

#[test]
fn test_model_unique_macro() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let _:() = client.del("UPerson:uniques:name").unwrap();

    let mut person = UPerson::default();
    assert_eq!(person.id, 0);
    assert_eq!(person.name, "".to_owned());
    assert_eq!(person.age, 18);
    assert_eq!(person.get_class_name(), "UPerson".to_owned());
    person.save(&client).unwrap();
    assert_eq!(person.unique_fields(), HashSet::from_iter(vec!["name"]));

    assert_eq!(get::<UPerson>(person.id, &client).unwrap(), person);
}
