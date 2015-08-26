#[macro_use(model, new)] extern crate ohmers;
extern crate redis;
extern crate rustc_serialize;

use ohmers::Ohmer;
use rustc_serialize::Encodable;

model!(Person {
        name:String = "".to_owned();
        age:u8 = 18;
        birthyear:u16 = 0;
        });

#[test]
fn test_new_macro() {
    let person = new!(Person {
            name: "John".to_owned(),
            birthyear: 1999,
            });

    assert_eq!(&*person.name, "John");
    assert_eq!(person.age, 18);
    assert_eq!(person.birthyear, 1999);
}
