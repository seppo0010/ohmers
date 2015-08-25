#[macro_use(incrby)] extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{all, all_query, Ohmer, Counter};
use redis::Commands;
use rustc_serialize::Encodable;

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
struct TvShow {
    id: usize,
    name: String,
    votes: Counter,
}

impl Default for TvShow {
    fn default() -> Self {
        TvShow {
            id: 0,
            name: "".to_string(),
            votes: Counter,
        }
    }
}
impl Ohmer for TvShow {
    fn id(&self) -> usize { self.id }
    fn set_id(&mut self, id: usize) { self.id = id; }
}

impl PartialEq for TvShow {
    fn eq(&self, other: &TvShow) -> bool {
        self.id == other.id
    }
}

macro_rules! create {
    ($name: ident, $votes: expr, $conn: expr) => {{
        let mut t = TvShow::default();
        t.name = stringify!($name).to_owned();
        t.save($conn).unwrap();
        incrby!(t, votes, $votes, $conn).unwrap();
        t
    }}
}
#[test]
fn test_sort() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let _:bool = client.del("TvShow:all").unwrap();

    // create it first to make it out of order
    let friends = create!(Friends, 45, &client);
    let scrubs = create!(Scrubs, 32, &client);

    let shows = [
        create!(ALF, 23, &client),
        create!(Community, 16, &client),
        create!(Damages, 9, &client),
        create!(Episodes, 36, &client),
        friends,
        create!(Fringe, 1, &client),
        scrubs,
        create!(Seinfeld, 33, &client),
        ];

    assert!(
            &*all::<TvShow>(&client).unwrap().collect::<Vec<_>>() !=
            &shows);
    assert_eq!(
            &*all_query::<TvShow>(&client).unwrap().sort(
                "name", None, true, true,
                ).unwrap().collect::<Vec<_>>(),
            &shows);
    assert_eq!(
            &*all_query::<TvShow>(&client).unwrap().sort(
                "votes", None, false, false,
                ).unwrap().collect::<Vec<_>>(),
            &[
            shows[4].clone(),
            shows[3].clone(),
            shows[7].clone(),
            shows[6].clone(),
            shows[0].clone(),
            shows[1].clone(),
            shows[2].clone(),
            shows[5].clone(),
            ]);
    assert_eq!(
            &*all_query::<TvShow>(&client).unwrap().sort(
                "votes", None, true, false,
                ).unwrap().collect::<Vec<_>>(),
            &[
            shows[5].clone(),
            shows[2].clone(),
            shows[1].clone(),
            shows[0].clone(),
            shows[6].clone(),
            shows[7].clone(),
            shows[3].clone(),
            shows[4].clone(),
            ]);
}
