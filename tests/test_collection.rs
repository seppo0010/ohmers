#[macro_use(model, create, collection)] extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{Ohmer, Reference, Collection};
use rustc_serialize::Encodable;

model!(
        derive { Clone }
        Year {
        year:u16 = 0;
        movies: Collection<Movie> = Collection::new();
        });

model!(
        derive { Clone }
        Movie {
            indices {
                year: Reference<Year> = Reference::new();
            };
            name:String = "".to_string();
        });

#[test]
fn test_movie_year() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let y85 = create!(Year { year: 1985, }, &client).unwrap();
    let y86 = create!(Year { year: 1986, }, &client).unwrap();

    let bttf = create!(Movie {
        name: "Back to the future".to_string(),
        year: Reference::with_value(&y85),
        }, &client).unwrap();

    let rocky_iv = create!(Movie {
        name: "Rocky IV".to_string(),
        year: Reference::with_value(&y85),
        }, &client).unwrap();

    let cocoon = create!(Movie {
        name: "Cocoon".to_string(),
        year: Reference::with_value(&y85),
        }, &client).unwrap();

    let _ = create!(Movie {
        name: "Top Gun".to_string(),
        year: Reference::with_value(&y86),
        }, &client).unwrap();

    let movies = y85.movies.all("year", &y85, &client).sort("name", None, true, true).unwrap().collect::<Vec<_>>();
    assert_eq!(
            movies,
            vec![bttf.clone(), cocoon.clone(), rocky_iv.clone()]
            );

    let movies = collection!(y85.movies, client).sort("name", None, true, true).unwrap().collect::<Vec<_>>();
    assert_eq!(
            movies,
            vec![bttf.clone(), cocoon.clone(), rocky_iv.clone()]
            );
}
