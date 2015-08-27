#[macro_use(insert)] extern crate ohmers;
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

impl Default for Team {
    fn default() -> Self {
        Team {
            id: 0,
            name: "".to_string(),
            players: Set::new(),
        }
    }
}
impl Ohmer for Team {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Player {
    id: usize,
    name: String,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            id: 0,
            name: "".to_string(),
        }
    }
}
impl Ohmer for Player {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

#[test]
fn test_set() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let mut p1 = Player::default();
    p1.name = "Alice".to_string();
    p1.save(&client).unwrap();

    let mut p2 = Player::default();
    p2.name = "Bob".to_string();
    p2.save(&client).unwrap();

    let mut team = Team::default();
    team.name = "foo".to_string();
    team.save(&client).unwrap();

    assert!(team.players.insert("players", &team, &p1, &client).unwrap());
    assert!(!team.players.insert("players", &team, &p1, &client).unwrap());
    assert!(insert!(team.players, &p2, &client).unwrap());
    assert!(!insert!(team.players, &p2, &client).unwrap());

    assert!(team.players.contains("players", &team, &p1, &client).unwrap());
    assert!(team.players.contains("players", &team, &p2, &client).unwrap());
    assert_eq!(team.players.len("players", &team, &client).unwrap(), 2);

    assert!(team.players.remove("players", &team, &p2, &client).unwrap());
    assert!(!team.players.remove("players", &team, &p2, &client).unwrap());

    assert!(team.players.contains("players", &team, &p1, &client).unwrap());
    assert!(!team.players.contains("players", &team, &p2, &client).unwrap());

    let players = team.players.query("players", &team, &client).unwrap().try_iter().unwrap().collect::<Vec<_>>();
    assert_eq!(players, vec![p1]);

    assert_eq!(team.players.len("players", &team, &client).unwrap(), 1);
}
