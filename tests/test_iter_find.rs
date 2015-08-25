extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use std::collections::HashSet;
use std::iter::FromIterator;

use ohmers::{all, Ohmer, Query};
use redis::Commands;
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Hash, Clone)]
struct Cat {
    id: usize,
    name: String,
    is_male: bool,
    age: u8,
}

impl Default for Cat {
    fn default() -> Self {
        Cat {
            id: 0,
            name: "".to_string(),
            is_male: false,
            age: 0,
        }
    }
}
impl Ohmer for Cat {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }

    fn index_fields<'a>(&self) -> HashSet<&'a str> {
        HashSet::from_iter(vec!["age", "is_male"].into_iter())
    }
}

#[test]
fn test_iter_find() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let _:bool = client.del("Cat:all").unwrap(); // oh, no!
    let _:bool = client.del("Cat:indices:is_male:0").unwrap();
    let _:bool = client.del("Cat:indices:is_male:1").unwrap();
    let _:bool = client.del("Cat:indices:age:7").unwrap();
    let _:bool = client.del("Cat:indices:age:2").unwrap();
    let _:bool = client.del("Cat:indices:age:3").unwrap();

    assert_eq!(
            all::<Cat>(&client).unwrap().collect::<HashSet<_>>().len(),
            0
            );

    let mut indiana = Cat::default();
    indiana.name = "Indiana".to_string();
    indiana.is_male = true;
    indiana.age = 7;
    indiana.save(&client).unwrap();

    let mut merry = Cat::default();
    merry.name = "Merry".to_string();
    merry.is_male = false;
    merry.age = 3;
    merry.save(&client).unwrap();

    let mut mozart = Cat::default();
    mozart.name = "Mozart".to_string();
    mozart.is_male = true;
    mozart.age = 2;
    mozart.save(&client).unwrap();

    let mut dorothy = Cat::default();
    dorothy.name = "Dorothy".to_string();
    dorothy.is_male = false;
    dorothy.age = 2;
    dorothy.save(&client).unwrap();

    let cats = Query::<Cat>::find("age", "2", &client).inter("is_male", "0")
        .try_iter().unwrap().collect::<Vec<_>>();
    assert_eq!(cats.len(), 1);
    assert_eq!(cats[0], dorothy);

    let cats = Query::<Cat>::find("age", "2", &client).union("is_male", "0")
        .try_iter().unwrap().collect::<HashSet<_>>();
    assert_eq!(HashSet::from_iter(vec![
                merry.clone(), mozart.clone(), dorothy.clone()].into_iter()),
            cats);

    let cats = Query::<Cat>::find("age", "2", &client).diff("is_male", "0")
        .try_iter().unwrap().collect::<HashSet<_>>();
    assert_eq!(HashSet::from_iter(vec![mozart.clone()].into_iter()), cats);

    let mut query = Query::<Cat>::find("age", "2", &client);
    query.diff("is_male", "0");
    let cats = query.try_into_iter().unwrap().collect::<HashSet<_>>();
    assert_eq!(HashSet::from_iter(vec![mozart].into_iter()), cats);
}
