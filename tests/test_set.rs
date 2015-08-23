extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{Ohmer, Set};
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Team {
    id: usize,
    name: String,
    players: Set<Player>,
}

impl Ohmer for Team {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
    fn defaults() -> Self {
        Team {
            id: 0,
            name: "".to_string(),
            players: Set::new(),
        }
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Player {
    id: usize,
    name: String,
}

impl Ohmer for Player {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
    fn defaults() -> Self {
        Player {
            id: 0,
            name: "".to_string(),
        }
    }
}

#[test]
fn test_set() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let mut p1 = Player::defaults();
    p1.name = "Alice".to_string();
    p1.save(&client).unwrap();

    let mut p2 = Player::defaults();
    p2.name = "Bob".to_string();
    p2.save(&client).unwrap();

    let mut team = Team::defaults();
    team.name = "foo".to_string();
    team.save(&client).unwrap();

    assert!(team.players.insert("players", &team, &p1, &client).unwrap());
    assert!(!team.players.insert("players", &team, &p1, &client).unwrap());
    assert!(team.players.insert("players", &team, &p2, &client).unwrap());
    assert!(!team.players.insert("players", &team, &p2, &client).unwrap());
    assert!(team.players.remove("players", &team, &p2, &client).unwrap());
    assert!(!team.players.remove("players", &team, &p2, &client).unwrap());

    let players = team.players.query("players", &team, &client).unwrap().try_iter().unwrap().collect::<Vec<_>>();
    assert_eq!(players, vec![p1]);
}
