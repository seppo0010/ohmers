extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{Ohmer, Counter, OhmerError};
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Candidate {
    id: usize,
    positive_votes: Counter,
    negative_votes: Counter,
}

impl Ohmer for Candidate {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
    fn defaults() -> Self {
        Candidate {
            id: 0,
            positive_votes: Counter,
            negative_votes: Counter,
        }
    }
}

#[test]
fn test_counter() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut candidate = Candidate::defaults();
    assert_eq!(candidate.positive_votes.incr(&candidate, "positive_votes", 1, &client).unwrap_err(), OhmerError::NotSaved);
    candidate.save(&client).unwrap();
    assert_eq!(candidate.positive_votes.incr(&candidate, "positive_votes", 1, &client).unwrap(), 1);
    assert_eq!(candidate.positive_votes.incr(&candidate, "positive_votes", 1, &client).unwrap(), 2);

    assert_eq!(candidate.positive_votes.get(&candidate, "positive_votes", &client).unwrap(), 2);
    assert_eq!(candidate.positive_votes.get(&candidate, "positive_votes", &client).unwrap(), 2);
    assert_eq!(candidate.negative_votes.get(&candidate, "negative_votes", &client).unwrap(), 0);
}
