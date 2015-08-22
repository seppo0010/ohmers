extern crate rmp as msgpack;
extern crate redis;
extern crate rustc_serialize;

use std::collections::HashMap;
use std::marker::PhantomData;

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

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct Reference<T: Ohmer> {
    id: usize,
    phantom: PhantomData<T>,
}

impl<T: Ohmer> Reference<T> {
    pub fn new() -> Self {
        Reference { id: 0, phantom: PhantomData }
    }

    pub fn get(&self, r: &redis::Client) -> Result<T, DecoderError> {
        get(self.id, r)
    }

    pub fn set(&mut self, obj: &T) {
        self.id = obj.id();
    }
}

#[derive(Debug, PartialEq)]
pub enum OhmerError {
    NotSaved,
    RedisError(redis::RedisError),
}

impl From<redis::RedisError> for OhmerError {
    fn from(e: redis::RedisError) -> OhmerError {
        OhmerError::RedisError(e)
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct Counter;

impl Counter {
    fn get_key<T: Ohmer>(&self, obj: &T, prop: &str) -> Result<String, OhmerError> {
        let class_name = obj.get_class_name();
        let id = obj.id();
        if id == 0 {
            return Err(OhmerError::NotSaved);
        }
        Ok(format!("{}:{}:{}", class_name, id, prop))
    }

    pub fn incr<T: Ohmer>(&self, obj: &T, prop: &str, incr: i64, r: &redis::Client) -> Result<i64, OhmerError> {
        let key = try!(self.get_key(obj, prop));
        Ok(try!(r.incr(key, incr)))
    }

    pub fn get<T: Ohmer>(&self, obj: &T, prop: &str, r: &redis::Client) -> Result<i64, OhmerError> {
        let key = try!(self.get_key(obj, prop));
        let r:Option<i64> = try!(r.get(key));
        match r {
            Some(v) => Ok(v),
            None => Ok(0),
        }
    }
}
