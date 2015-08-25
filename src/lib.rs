extern crate rmp as msgpack;
extern crate redis;
extern crate rustc_serialize;
extern crate regex;
extern crate stal;

use std::collections::{HashSet, HashMap};
use std::marker::PhantomData;
use std::mem::replace;
use std::string::FromUtf8Error;

use redis::Commands;
use redis::ToRedisArgs;
use regex::Regex;

mod encoder;
use encoder::*;

mod decoder;
use decoder::*;

mod save;
use save::SAVE;

#[macro_export]
macro_rules! model {
    ($class: ident, $($key: ident => ($proptype: ty, $default: expr)),* ) => {
        #[derive(RustcEncodable, RustcDecodable, Debug)]
        struct $class {
            id: usize,
            $(
                $key: $proptype,
            )*
        }

        impl ohmers::Ohmer for $class {
            fn id(&self) -> usize { self.id }
            fn set_id(&mut self, id: usize) { self.id = id; }
            fn defaults() -> Self {
                $class {
                    id: 0,
                    $(
                        $key: $default,
                    )*
                }
            }

            // These functions are implemented in the trait, but this
            // reduces the runtime overhead
            fn get_class_name(&self) -> String {
                stringify!($class).to_owned()
            }

            fn key_for_unique(&self, field: &str, value: &str) -> String {
                format!("{}:uniques:{}:{}", stringify!($class), field, value)
            }

            fn key_for_index(&self, field: &str, value: &str) -> String {
                format!("{}:indices:{}:{}", stringify!($class), field, value)
            }
        }

        impl PartialEq for $class {
            fn eq(&self, other: &$class) -> bool {
                self.id == other.id
            }
        }
    }
}

pub fn with<T: Ohmer, S: ToRedisArgs>(property: &str, value: S, r: &redis::Client) -> Result<Option<T>, DecoderError> {
    let mut obj = T::defaults();

    let opt_id:Option<usize> = try!(r.hget(format!("{}:uniques:{}", obj.get_class_name(), property), value));

    let id = match opt_id {
        Some(id) => id,
        None => return Ok(None),
    };
    try!(obj.load(id, r));
    Ok(Some(obj))
}

pub fn get<T: Ohmer>(id: usize, r: &redis::Client) -> Result<T, DecoderError> {
    let mut obj = T::defaults();
    try!(obj.load(id, r));
    Ok(obj)
}

pub fn all_query<'a, T: 'a + Ohmer>(r: &'a redis::Client) -> Result<Query<'a, T>, OhmerError> {
    let class_name = T::defaults().get_class_name();
    Ok(Query::<'a, T>::new(stal::Set::Key(format!("{}:all", class_name).as_bytes().to_vec()), r))
}

pub fn all<'a, T: 'a + Ohmer>(r: &'a redis::Client) -> Result<Iter<T>, OhmerError> {
    Ok(try!(try!(all_query(r)).try_iter()))
}

pub trait Ohmer : rustc_serialize::Encodable + rustc_serialize::Decodable + Sized {
    fn id_field(&self) -> String { "id".to_string() }
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);

    fn defaults() -> Self;

    fn unique_fields<'a>(&self) -> HashSet<&'a str> { HashSet::new() }
    fn index_fields<'a>(&self) -> HashSet<&'a str> { HashSet::new() }

    fn key_for_unique(&self, field: &str, value: &str) -> String {
        format!("{}:uniques:{}:{}", self.get_class_name(), field, value)
    }

    fn key_for_index(&self, field: &str, value: &str) -> String {
        format!("{}:indices:{}:{}", self.get_class_name(), field, value)
    }

    fn counters(&self) -> HashSet<String> {
        let mut encoder = Encoder::new();
        self.encode(&mut encoder).unwrap();
        encoder.counters
    }

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

    fn save(&mut self, r: &redis::Client) -> Result<(), OhmerError>{
        let mut encoder = Encoder::new();
        encoder.id_field = self.id_field();
        try!(self.encode(&mut encoder));

        let mut unique_fields = self.unique_fields();
        let mut index_fields = self.index_fields();
        let mut uniques = HashMap::new();
        let mut indices = HashMap::new();

        for i in 0..(encoder.attributes.len() / 2) {
            let pos = i * 2;
            let key = &encoder.attributes[pos];
            if unique_fields.remove(&**key) {
                uniques.insert(key.clone(), encoder.attributes[pos + 1].clone());
            }
            if index_fields.remove(&**key) {
                indices.insert(key.clone(), vec![encoder.attributes[pos + 1].clone()]);
            }
        }
        if unique_fields.len() > 0 {
            return Err(OhmerError::UnknownIndex(unique_fields.iter().next().unwrap().to_string()));
        }
        if index_fields.len() > 0 {
            return Err(OhmerError::UnknownIndex(index_fields.iter().next().unwrap().to_string()));
        }

        let script = redis::Script::new(SAVE);
        let result = script
                .arg(try!(msgpack_encode(&encoder.features)))
                .arg(try!(msgpack_encode(&encoder.attributes.iter().map(|x| &*x).collect::<Vec<_>>())))
                .arg(try!(msgpack_encode(&indices)))
                .arg(try!(msgpack_encode(&uniques)))
                .invoke(&try!(r.get_connection()));
        let id = match result {
            Ok(id) => id,
            Err(e) => {
                let re = Regex::new(r"UniqueIndexViolation: (\w+)").unwrap();
                let s = format!("{}", e);
                match re.find(&*s) {
                    Some((start, stop)) => return Err(OhmerError::UniqueIndexViolation(s[start + 22..stop].to_string())),
                    None => return Err(OhmerError::RedisError(e)),
                }
            },
        };
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

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct Set<T: Ohmer> {
    phantom: PhantomData<T>,
}

impl<T: Ohmer> Set<T> {
    pub fn new() -> Self {
        Set { phantom: PhantomData }
    }

    fn key_name<P: Ohmer>(&self, property: &str, parent: &P) -> Result<String, OhmerError> {
        let id = parent.id();
        if id == 0 {
            Err(OhmerError::NotSaved)
        } else {
            Ok(format!("{}:{}:{}", parent.get_class_name(), property, parent.id()))
        }
    }

    pub fn key<P: Ohmer>(&self, property: &str, parent: &P) -> Result<stal::Set, OhmerError> {
        Ok(stal::Set::Key(try!(self.key_name(property, parent)).as_bytes().to_vec()))
    }

    pub fn query<'a, P: Ohmer>(&'a self, property: &str, parent: &P, r: &'a redis::Client) -> Result<Query<T>, OhmerError> {
        let key = try!(self.key(property, parent));
        Ok(Query::new(key, r))
    }

    pub fn insert<P: Ohmer>(&self, property: &str, parent: &P, obj: &T, r: &redis::Client) -> Result<bool, OhmerError> {
        Ok(try!(r.sadd(try!(self.key_name(property, parent)), obj.id())))
    }

    pub fn remove<P: Ohmer>(&self, property: &str, parent: &P, obj: &T, r: &redis::Client) -> Result<bool, OhmerError> {
        Ok(try!(r.srem(try!(self.key_name(property, parent)), obj.id())))
    }

    pub fn contains<P: Ohmer>(&self, property: &str, parent: &P, obj: &T, r: &redis::Client) -> Result<bool, OhmerError> {
        Ok(try!(r.sismember(try!(self.key_name(property, parent)), obj.id())))
    }

    pub fn len<P: Ohmer>(&self, property: &str, parent: &P, r: &redis::Client) -> Result<usize, OhmerError> {
        Ok(try!(r.scard(try!(self.key_name(property, parent)))))
    }
}

#[derive(PartialEq, Debug)]
pub enum OhmerError {
    NotSaved,
    RedisError(redis::RedisError),
    EncoderError(EncoderError),
    UnknownIndex(String),
    UniqueIndexViolation(String),
    CommandError(Vec<u8>),
}

impl From<FromUtf8Error> for OhmerError {
    fn from(err: FromUtf8Error) -> OhmerError {
        OhmerError::CommandError(err.into_bytes())
    }
}

impl From<redis::RedisError> for OhmerError {
    fn from(e: redis::RedisError) -> OhmerError {
        OhmerError::RedisError(e)
    }
}

impl From<EncoderError> for OhmerError {
    fn from(e: EncoderError) -> OhmerError {
        OhmerError::EncoderError(e)
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
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

#[macro_export]
macro_rules! incrby {
    ($obj: expr, $prop: ident, $incr: expr, $client: expr) => {{
        $obj.$prop.incr(&$obj, stringify!($prop), $incr, $client)
    }}
}

#[macro_export]
macro_rules! incr {
    ($obj: expr, $prop: ident, $client: expr) => {{
        incrby!($obj, $prop, 1, $client)
    }}
}

#[macro_export]
macro_rules! decr {
    ($obj: expr, $prop: ident, $client: expr) => {{
        incrby!($obj, $prop, -1, $client)
    }}
}

pub struct Query<'a, T: 'a + Ohmer> {
    set: stal::Set,
    r: &'a redis::Client,
    phantom: PhantomData<T>,
}

impl<'a, T: Ohmer> Query<'a, T> {
    fn new(set: stal::Set, r: &'a redis::Client) -> Self {
        Query { set: set, phantom: PhantomData, r: r }
    }

    fn key(field: &str, value: &str) -> stal::Set {
        stal::Set::Key(T::defaults().key_for_index(field, value).as_bytes().to_vec())
    }

    pub fn find(field: &str, value: &str, r: &'a redis::Client) -> Self {
        Query { set: Query::<T>::key(field, value), phantom: PhantomData, r: r }
    }

    pub fn inter(&mut self, field: &str, value: &str) -> &mut Self {
        self.sinter(vec![Query::<T>::key(field, value)]);
        self
    }

    pub fn sinter(&mut self, mut sets: Vec<stal::Set>) {
        let set = replace(&mut self.set, stal::Set::Key(vec![]));
        sets.push(set);
        self.set = stal::Set::Inter(sets);
    }

    pub fn union(&mut self, field: &str, value: &str) -> &mut Self {
        self.sunion(vec![Query::<T>::key(field, value)]);
        self
    }

    pub fn sunion(&mut self, mut sets: Vec<stal::Set>) {
        let set = replace(&mut self.set, stal::Set::Key(vec![]));
        sets.push(set);
        self.set = stal::Set::Union(sets);
    }

    pub fn diff(&mut self, field: &str, value: &str) -> &mut Self {
        self.sdiff(vec![Query::<T>::key(field, value)]);
        self
    }

    pub fn sdiff(&mut self, mut sets: Vec<stal::Set>) {
        let set = replace(&mut self.set, stal::Set::Key(vec![]));
        sets.insert(0, set);
        self.set = stal::Set::Diff(sets);
    }

    pub fn try_iter(&self) -> Result<Iter<'a, T>, OhmerError> {
        Iter::new(self.set.ids().solve(), self.r)
    }

    pub fn try_into_iter(self) -> Result<Iter<'a, T>, OhmerError> {
        Iter::new(self.set.into_ids().solve(), self.r)
    }

    pub fn sort(&self, by: &str, limit: Option<(usize, usize)>, asc: bool, alpha: bool) -> Result<Iter<'a, T>, OhmerError> {
        let default = T::defaults();
        let class_name = default.get_class_name();
        let key = if default.counters().contains(by) {
            format!("{}:*:{}", class_name, by)
        } else {
            format!("{}:*->{}", class_name, by)
        }.as_bytes().to_vec();

        let mut template = vec![b"SORT".to_vec(), vec![], b"BY".to_vec(), key];
        if let Some(l) = limit {
            template.push(b"LIMIT".to_vec());
            template.push(format!("{}", l.0).as_bytes().to_vec());
            template.push(format!("{}", l.1).as_bytes().to_vec());
        }
        template.push(if asc { b"ASC".to_vec() } else { b"DESC".to_vec() });
        if alpha {
            template.push(b"ALPHA".to_vec());
        }

        let stal = stal::Stal::from_template(template, vec![(self.set.clone(), 1)]);
        Iter::new(stal.solve(), self.r)
    }
}

pub struct Iter<'a, T> {
    r: &'a redis::Client,
    iter: std::vec::IntoIter<usize>,
    phantom: PhantomData<T>,
}

impl<'a, T: Ohmer> Iter<'a, T> {
    fn new(ops: (Vec<Vec<Vec<u8>>>, usize), r: &'a redis::Client) -> Result<Self, OhmerError> {
        let mut q = redis::pipe();
        q.atomic();
        let mut i = 0;
        let len = ops.0.len();

        for op in ops.0.into_iter() {
            if i == 0 || i == len - 1 {
                i += 1;
                // skip MULTI and EXEC
                continue;
            }
            let mut first = true;
            for arg in op {
                if first {
                    q.cmd(&*try!(String::from_utf8(arg)));
                    first = false;
                } else {
                    q.arg(arg);
                }
                if i != ops.1 {
                    q.ignore();
                }
            }
            i += 1;
        }
        let mut result:Vec<Vec<usize>> = try!(q.query(r));
        Ok(Iter { iter: result.pop().unwrap().into_iter(), r: r, phantom: PhantomData })
    }
}

impl<'a, T: Ohmer> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.iter.next() {
            Some(id) => match get(id, self.r) {
                Ok(v) => Some(v),
                Err(e) => { println!("AAA {:?}", e); None},
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len(), Some(self.iter.len()))
    }
}
