#[macro_use(model, create, find)] extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::Ohmer;
use redis::Commands;
use rustc_serialize::Encodable;

model!(derive { Clone } IPerson {
        uniques {};
        indices {
            age:u8 = 18;
            month_of_birth:u8 = 0;
            day_of_birth:u8 = 0;
        };
        name:String = "".to_owned();
        });

#[test]
fn test_model_find_macro() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let _:bool = client.del("IPerson:indices:age:18").unwrap();
    let _:bool = client.del("IPerson:indices:month_of_birth:1").unwrap();
    let _:bool = client.del("IPerson:indices:month_of_birth:2").unwrap();
    let _:bool = client.del("IPerson:indices:month_of_birth:3").unwrap();
    let _:bool = client.del("IPerson:indices:day_of_birth:1").unwrap();
    let _:bool = client.del("IPerson:indices:day_of_birth:2").unwrap();
    let _:bool = client.del("IPerson:indices:day_of_birth:3").unwrap();

    let john = create!(IPerson {
            month_of_birth: 1,
            day_of_birth: 1,
            name: "John".to_string(),
            }, &client).unwrap();

    let _ = create!(IPerson {
            month_of_birth: 1,
            day_of_birth: 2,
            name: "Jane".to_string(),
            }, &client).unwrap();

    let alice = create!(IPerson {
            month_of_birth: 1,
            day_of_birth: 3,
            name: "Alice".to_string(),
            }, &client).unwrap();

    let bob = create!(IPerson {
            month_of_birth: 2,
            day_of_birth: 1,
            name: "Bob".to_string(),
            }, &client).unwrap();

    assert_eq!(
            find!(IPerson {
                day_of_birth: 3,
            }, &client).try_into_iter().unwrap().collect::<Vec<IPerson>>(),
            vec![alice.clone()]
            );

    assert_eq!(
            find!(IPerson {
                day_of_birth: 1,
            }, &client).sort("name", None, true, true).unwrap().collect::<Vec<IPerson>>(),
            vec![bob.clone(), john.clone()]
            );
    assert_eq!(
            find!(IPerson {
                day_of_birth: 1,
            }, &client).sort("name", None, true, true).unwrap().collect::<Vec<IPerson>>(),
            vec![bob.clone(), john.clone()]
            );

    assert_eq!(
            find!(IPerson {
                day_of_birth: 3,
            } || {
                month_of_birth: 2,
            }, &client).sort("name", None, true, true).unwrap().collect::<Vec<IPerson>>(),
            vec![alice.clone(), bob.clone()]
            );
}
