#[macro_use(model, create, incr)] extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::{Counter, Ohmer};
use redis::Commands;
use rustc_serialize::Encodable;

model!(derive { Clone } Chair {
        uniques {
            location:u32 = 0;
        };
        indices {
            color:u32 = 0;
        };
        legs:Counter = Counter;
        });

#[test]
fn test_model_delete() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    for key in client.scan_match::<_, String>("*Chair*").unwrap().into_iter() {
        let _:bool = client.del(key).unwrap();
    }

    let chair1 = create!(Chair {
            color: 0x0000FF,
            location: 1,
            }, &client).unwrap();
    incr!(chair1.legs, &client).unwrap();
    chair1.delete(&client).unwrap();
    assert_eq!(
            vec!["Chair:id".to_owned()],
            client.scan_match::<_, String>("*Chair*").unwrap().into_iter().collect::<Vec<_>>()
            );
}
