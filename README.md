# ohmers

[![Build Status](https://travis-ci.org/seppo0010/ohmers.svg?branch=master)](https://travis-ci.org/seppo0010/ohmers)
[![crates.io](http://meritbadge.herokuapp.com/ohmers)](https://crates.io/crates/ohmers)

A library for retrieving and storing objects in a Redis server.

The crate is called `ohmers` and you can depend on it via cargo:

```toml
[dependencies]
ohmers = "0.1.0"
```

## Example

```rust
#[macro_use(model, create, insert)] extern crate ohmers;
extern crate rustc_serialize;
extern crate redis;
use ohmers::*;

model!(Event {
    indices {
        name:String = "My Event".to_string();
    };
    venue:Reference<Venue> = Reference::new();
    participants:Set<Person> = Set::new();
    votes:Counter = Counter;
});

model!(Venue {
    name:String = "My Venue".to_string();
    events:Set<Event> = Set::new();
});

model!(Person {
    name:String = "A Person".to_string();
});

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let p1 = create!(Person { name: "Alice".to_string(), }, &client).unwrap();
    let p2 = create!(Person { name: "Bob".to_string(), }, &client).unwrap();
    let p3 = create!(Person { name: "Charlie".to_string(), }, &client).unwrap();

    let v1 = create!(Venue { name: "Home".to_string(), }, &client).unwrap();
    let v2 = create!(Venue { name: "Work".to_string(), }, &client).unwrap();

    let mut e1 = create!(Event { name: "Birthday Party".to_string(), }, &client).unwrap();
    insert!(e1.participants, p1, &client).unwrap();
    insert!(e1.participants, p2, &client).unwrap();
    insert!(e1.participants, p3, &client).unwrap();
    e1.venue.set(&v1);
    e1.save(&client).unwrap();

    let mut e2 = create!(Event { name: "Work Meeting".to_string(), }, &client).unwrap();
    insert!(e2.participants, p1, &client).unwrap();
    insert!(e2.participants, p2, &client).unwrap();
    e2.venue.set(&v2);
    e2.save(&client).unwrap();
}
```

## Documentation

For a more comprehensive documentation with all the available functions and
parameters go to http://seppo0010.github.io/ohmers/
