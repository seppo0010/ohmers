extern crate rmp as msgpack;
extern crate redis;
extern crate rustc_serialize;

use std::collections::HashMap;

use redis::Commands;

mod encoder;
use encoder::*;

mod decoder;
use decoder::*;

mod save;
use save::SAVE;

pub fn get<T: Ohmer>(id: usize, r: &redis::Client) -> Result<T, DecoderError> {
    let mut obj = T::defaults();
    try!(obj.load(id, r));
    Ok(obj)
}

pub trait Ohmer : rustc_serialize::Encodable + rustc_serialize::Decodable + Sized {
    fn id_field(&self) -> String { "id".to_string() }
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);

    fn defaults() -> Self;

    fn get_class_name(&self) -> String {
        let mut encoder = Encoder::new();
        self.encode(&mut encoder).unwrap();
        encoder.features.remove("name").unwrap()
    }

    fn load(&mut self, id: usize, r: &redis::Client) -> Result<(), DecoderError> {
        let mut properties:HashMap<String, String> = try!(try!(r.get_connection()).hgetall(format!("{}:{}", self.get_class_name(), id)));
        properties.insert("id".to_string(), format!("{}", id));

        let mut decoder = Decoder::new(properties);
        *self = try!(rustc_serialize::Decodable::decode(&mut decoder));
        Ok(())
    }

    fn save(&mut self, r: &redis::Client) -> Result<(), EncoderError>{
        let mut encoder = Encoder::new();
        encoder.id_field = self.id_field();
        try!(self.encode(&mut encoder));
        let script = redis::Script::new(SAVE);
        let id = try!(script
                .arg(try!(msgpack_encode(&encoder.features)))
                .arg(try!(msgpack_encode(&encoder.attributes.iter().map(|x| &*x).collect::<Vec<_>>())))
                .arg(try!(msgpack_encode(&Vec::new() as &Vec<u8>)))
                .arg(try!(msgpack_encode(&Vec::new() as &Vec<u8>)))
                .invoke(&try!(r.get_connection())));
        self.set_id(id);
        Ok(())
    }
}
