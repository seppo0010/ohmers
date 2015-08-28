//! Object-hash mapping library for Redis.
//!
//! Ohmers is a library for storing objects in Redis, a persistent
//! key-value database.
//! It is based on the Ruby library Ohm, and it uses the same key names,
//! so it can be used in the same system.
//!
//! # Prerequisites
//!
//! Have a [redis server](https://github.com/antirez/redis/) running and a
//! [redis-rs](https://github.com/mitsuhiko/redis-rs/) connection.
//!
//! # Getting started
//!
//! Ohmers maps Rust structs to hash maps in Redis. First define the structs
//! using the model! macro, and then use their methods to created, read,
//! update, delete.
//!
//! ```rust
//! # #[macro_use(model, create, insert)] extern crate ohmers;
//! # extern crate rustc_serialize;
//! # extern crate redis;
//! # use ohmers::*;
//!
//! model!(Event {
//!     indices {
//!         name:String = "My Event".to_string();
//!     };
//!     venue:Reference<Venue> = Reference::new();
//!     participants:Set<Person> = Set::new();
//!     votes:Counter = Counter;
//! });
//!
//! model!(Venue {
//!     name:String = "My Venue".to_string();
//!     events:Set<Event> = Set::new();
//! });
//!
//! model!(Person {
//!     name:String = "A Person".to_string();
//! });
//! # fn main() {
//! # let client = redis::Client::open("redis://127.0.0.1/").unwrap();
//! let p1 = create!(Person { name: "Alice".to_string(), }, &client).unwrap();
//! let p2 = create!(Person { name: "Bob".to_string(), }, &client).unwrap();
//! let p3 = create!(Person { name: "Charlie".to_string(), }, &client).unwrap();
//!
//! let v1 = create!(Venue { name: "Home".to_string(), }, &client).unwrap();
//! let v2 = create!(Venue { name: "Work".to_string(), }, &client).unwrap();
//!
//! let mut e1 = create!(Event { name: "Birthday Party".to_string(), }, &client).unwrap();
//! insert!(e1.participants, p1, &client).unwrap();
//! insert!(e1.participants, p2, &client).unwrap();
//! insert!(e1.participants, p3, &client).unwrap();
//! e1.venue.set(&v1);
//! e1.save(&client).unwrap();
//!
//! let mut e2 = create!(Event { name: "Work Meeting".to_string(), }, &client).unwrap();
//! insert!(e2.participants, p1, &client).unwrap();
//! insert!(e2.participants, p2, &client).unwrap();
//! e2.venue.set(&v2);
//! e2.save(&client).unwrap();
//! # }
//! ```
pub extern crate rmp as msgpack;
extern crate redis;
extern crate rustc_serialize;
extern crate regex;
extern crate stal;

use std::ascii::AsciiExt;
use std::collections::{HashSet, HashMap};
use std::marker::PhantomData;
use std::mem::replace;
use std::string::FromUtf8Error;

use redis::Commands;
use redis::ToRedisArgs;
use regex::Regex;
pub use stal::Set as StalSet;

mod encoder;
use encoder::*;

mod decoder;
use decoder::*;

mod lua;
use lua::{DELETE, SAVE};

/// Declares a struct.
/// Fields may be declared as a part of uniques, indices, or regular fields.
/// Every field must have a default value.
/// The struct will derive RustcEncodable, RustcDecodable, and Default.
/// More `derive`s can be specified.
///
/// A property `id: usize = 0;` is automatically added to track the object.
///
/// # Examples
/// ```
/// # #[macro_use(model)] extern crate ohmers;
/// # extern crate rustc_serialize;
/// model!(
///     derive { Clone, PartialOrd }
///     MyStruct {
///         uniques { my_unique_identifier:u8 = 0; };
///         indices { my_index:u8 = 0; };
///         other_field:String = "".to_string();
///     });
/// # fn main() {
/// # }
/// ```
#[macro_export]
macro_rules! model {
    ($class: ident { $($key: ident:$proptype: ty = $default: expr);*; } ) => {
        model!(
                $class {
                    uniques { };
                    indices { };
                    $($key:$proptype = $default;)*
                }
                );
    };
    (
     derive { $($derive: ident),* }
     $class: ident { $($key: ident:$proptype: ty = $default: expr);*; } ) => {
        model!(
                derive { $($derive),* }
                $class {
                    uniques { };
                    indices { };
                    $($key:$proptype = $default;)*
                }
                );
    };
    ($class: ident {
     uniques { $($ukey: ident:$uproptype: ty = $udefault: expr;)* };
     $($key: ident:$proptype: ty = $default: expr;)* }
     ) => {
        model!(
                $class {
                    uniques {
                        $(
                            $ukey: $uproptype = $udefault;
                        )*
                    };
                    indices { };
                    $($key:$proptype = $default;)*
                }
                );
    };
    (
     derive { $($derive: ident),* }
     $class: ident {
     uniques { $($ukey: ident:$uproptype: ty = $udefault: expr;)* };
     $($key: ident:$proptype: ty = $default: expr;)* }
     ) => {
        model!(
                derive { $($derive),* }
                $class {
                    uniques {
                        $(
                            $ukey: $uproptype = $udefault;
                        )*
                    };
                    indices { };
                    $($key:$proptype = $default;)*
                }
                );
    };
    ($class: ident {
     indices { $($ikey: ident:$iproptype: ty = $idefault: expr;)* };
     $($key: ident:$proptype: ty = $default: expr;)* }
     ) => {
        model!(
                $class {
                    uniques { };
                    indices {
                        $(
                            $ikey: $iproptype = $idefault;
                        )*
                    };
                    $($key:$proptype = $default;)*
                }
                );
    };
    (
     derive { $($derive: ident),* }
     $class: ident {
     indices { $($ikey: ident:$iproptype: ty = $idefault: expr;)* };
     $($key: ident:$proptype: ty = $default: expr;)* }
     ) => {
        model!(
                derive { $($derive),* }
                $class {
                    uniques { };
                    indices {
                        $(
                            $ikey: $iproptype = $idefault;
                        )*
                    };
                    $($key:$proptype = $default;)*
                }
                );
    };
    (
     $class: ident {
     uniques { $($ukey: ident:$uproptype: ty = $udefault: expr;)* };
     indices { $($ikey: ident:$iproptype: ty = $idefault: expr;)* };
     $($key: ident:$proptype: ty = $default: expr;)* }
     ) => {
        model!(
                derive { }
                $class {
                    uniques {
                        $(
                            $ukey: $uproptype = $udefault;
                        )*
                    };
                    indices {
                        $(
                            $ikey: $iproptype = $idefault;
                        )*
                    };
                    $($key:$proptype = $default;)*
                }
                );
    };
    (
     derive { $($derive: ident),* }
     $class: ident {
     uniques { $($ukey: ident:$uproptype: ty = $udefault: expr;)* };
     indices { $($ikey: ident:$iproptype: ty = $idefault: expr;)* };
     $($key: ident:$proptype: ty = $default: expr;)* }
     ) => {
        #[derive(RustcEncodable, RustcDecodable, Debug, $($derive,)* )]
        struct $class {
            id: usize,
            $(
                $key: $proptype,
            )*
            $(
                $ukey: $uproptype,
            )*
            $(
                $ikey: $iproptype,
            )*
        }

        impl Default for $class {
            fn default() -> Self {
                $class {
                    id: 0,
                    $(
                        $key: $default,
                    )*
                    $(
                        $ukey: $udefault,
                    )*
                    $(
                        $ikey: $idefault,
                    )*
                }
            }
        }

        impl ohmers::Ohmer for $class {
            fn id(&self) -> usize { self.id }
            fn set_id(&mut self, id: usize) { self.id = id; }

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

            fn unique_fields<'a>(&self) -> std::collections::HashSet<&'a str> {
                #![allow(unused_mut)]
                let mut hs = std::collections::HashSet::new();
                $(
                    hs.insert(stringify!($ukey));
                )*
                hs
            }

            fn index_fields<'a>(&self) -> std::collections::HashSet<&'a str> {
                #![allow(unused_mut)]
                let mut hs = std::collections::HashSet::new();
                $(
                    hs.insert(stringify!($ikey));
                )*
                hs
            }
        }

        impl PartialEq for $class {
            fn eq(&self, other: &$class) -> bool {
                self.id == other.id
            }
        }
    }
}

/// Creates a new instance of `$class` using the default properties,
/// overriding specified collection of `$key` with `$value`.
///
/// # Examples
/// ```
/// # #[macro_use(model, new)] extern crate ohmers;
/// # extern crate rustc_serialize;
/// model!(
///     MyStruct {
///         k1:u8 = 1;
///         k2:u8 = 2;
///     });
///
/// # fn main() {
/// let st = new!(MyStruct { k2: 3, });
/// assert_eq!(st.id, 0); // object was not created in Redis yet
/// assert_eq!(st.k1, 1);
/// assert_eq!(st.k2, 3);
/// # }
/// ```
#[macro_export]
macro_rules! new {
    ($class: ident { $($key:ident: $value: expr),*, }) => {{
        let mut obj = $class::default();
        $(
            obj.$key = $value;
        )*
        obj
    }}
}

/// Creates a new instance of `$class` using the default properties,
/// overriding specified collection of `$key` with `$value`, and saving it
/// in the database
///
/// # Examples
/// ```
/// # #[macro_use(model, create)] extern crate ohmers;
/// # extern crate rustc_serialize;
/// # extern crate redis;
/// # use ohmers::Ohmer;
/// model!(
///     MyStruct {
///         k1:u8 = 1;
///         k2:u8 = 2;
///     });
///
/// # fn main() {
/// # let client = redis::Client::open("redis://127.0.0.1/").unwrap();
/// let st = create!(MyStruct { k2: 3, }, &client).unwrap();
/// assert!(st.id > 0); // object was already created in Redis
/// assert_eq!(st.k1, 1);
/// assert_eq!(st.k2, 3);
/// # }
/// ```
#[macro_export]
macro_rules! create {
    ($class: ident { $($key:ident: $value: expr),*, }, $conn: expr) => {{
        let mut obj = $class::default();
        $(
            obj.$key = $value;
        )*
        obj.save(&$conn).map(|_| obj)
    }}
}

/// Returns a `Query` with all the `$class` objects  where `$key` is `$value`.
/// All the `$key` must be declared as `indices` in the `model!` declaration.
///
/// # Examples
///
/// ```rust
/// # #[macro_use(model, create, find)] extern crate ohmers;
/// # extern crate rustc_serialize;
/// # extern crate redis;
/// # use ohmers::Ohmer;
/// # use redis::Commands;
/// model!(
///     Browser {
///         indices {
///             name:String = "".to_string();
///             major_version:u8 = 0;
///         };
///         minor_version:u8 = 0;
///     });
///
/// # fn main() {
/// # let client = redis::Client::open("redis://127.0.0.1/").unwrap();
/// # let _:bool = client.del("Browser:indices:name:Firefox").unwrap();
/// # let _:bool = client.del("Browser:indices:name:Chrome").unwrap();
/// # let _:bool = client.del("Browser:indices:major_version:42").unwrap();
/// # let _:bool = client.del("Browser:indices:major_version:43").unwrap();
/// # let _:bool = client.del("Browser:indices:major_version:44").unwrap();
/// create!(Browser { name: "Firefox".to_string(), major_version: 42, minor_version: 3, }, &client).unwrap();
/// create!(Browser { name: "Firefox".to_string(), major_version: 42, minor_version: 4, }, &client).unwrap();
/// create!(Browser { name: "Firefox".to_string(), major_version: 43, }, &client).unwrap();
/// create!(Browser { name: "Firefox".to_string(), major_version: 43, minor_version: 1, }, &client).unwrap();
/// create!(Browser { name: "Chrome".to_string(), major_version: 43, minor_version: 1, }, &client).unwrap();
/// create!(Browser { name: "Chrome".to_string(), major_version: 43, minor_version: 2, }, &client).unwrap();
/// create!(Browser { name: "Chrome".to_string(), major_version: 44, minor_version: 3, }, &client).unwrap();
///
/// assert_eq!(find!(
///     Browser { name: "Chrome", major_version: 44, } ||
///     { name: "Firefox", major_version: 43, },
///     &client
/// ).try_into_iter().unwrap().collect::<Vec<_>>().len(), 3);
/// # }
/// ```
#[macro_export]
macro_rules! find {
    ($class: ident $({ $($key:ident: $value: expr),*, })||*, $conn: expr) => {{
        ohmers::Query::<$class>::new(
                ohmers::StalSet::Union(vec![
                    $(
                    ohmers::StalSet::Inter(
                        vec![
                        $(
                            ohmers::Query::<$class>::key(stringify!($key), &*format!("{}", $value)),
                        )*
                        ]
                    ),
                    )*
                    ]
                ), &$conn)
    }}
}

/// Properties declared as `Collection` can use the collection macro to get a
/// `Query` to iterate over all of its elements.
/// A `Collection` is an accessor to objects that have a `Reference` to the
/// object.
#[macro_export]
macro_rules! collection {
    ($obj: ident.$prop: ident, $conn: expr) => {{
        $obj.$prop.all(&*$obj.get_class_name(), &$obj, &$conn)
    }}
}

/// Number of elements in a List or Set property.
#[macro_export]
macro_rules! len {
    ($obj: ident. $prop: ident, $conn: expr) => {{
        $obj.$prop.len(stringify!($prop), &$obj, &$conn)
    }}
}

/// Insert `$el` in `$obj.$prop`. The property must be a Set.
#[macro_export]
macro_rules! insert {
    ($obj: ident.$prop: ident, $el: expr, $conn: expr) => {{
        $obj.$prop.insert(stringify!($prop), &$obj, &$el, &$conn)
    }}
}

/// Adds `$el` at the end of `$obj.$prop`. The property must be a List.
#[macro_export]
macro_rules! push_back {
    ($obj: ident.$prop: ident, $el: expr, $conn: expr) => {{
        $obj.$prop.push_back(stringify!($prop), &$obj, &$el, &$conn)
    }}
}

/// Adds `$el` at the beginning of `$obj.$prop`. The property must be a List.
#[macro_export]
macro_rules! push_front {
    ($obj: ident.$prop: ident, $el: expr, $conn: expr) => {{
        $obj.$prop.push_front(stringify!($prop), &$obj, &$el, &$conn)
    }}
}

/// Retrieves and remove an element from the end of `$obj.$prop`.
/// The property must be a List.
#[macro_export]
macro_rules! pop_back {
    ($obj: ident.$prop: ident, $conn: expr) => {{
        $obj.$prop.pop_back(stringify!($prop), &$obj, &$conn)
    }}
}

/// Retrieves and remove an element from the beginning of `$obj.$prop`.
/// The property must be a List.
#[macro_export]
macro_rules! pop_front {
    ($obj: ident.$prop: ident, $conn: expr) => {{
        $obj.$prop.pop_front(stringify!($prop), &$obj, &$conn)
    }}
}

/// Retrieves an element from the beginning of `$obj.$prop`.
/// The property must be a List.
#[macro_export]
macro_rules! first {
    ($obj: ident.$prop: ident, $conn: expr) => {{
        $obj.$prop.first(stringify!($prop), &$obj, &$conn)
    }}
}

/// Retrieves an element from the end of `$obj.$prop`.
/// The property must be a List.
#[macro_export]
macro_rules! last {
    ($obj: ident.$prop: ident, $conn: expr) => {{
        $obj.$prop.last(stringify!($prop), &$obj, &$conn)
    }}
}

/// Creates an iterable of `$obj.$prop` between `$start` and `$end`.
/// The property must be a List.
///
/// # Examples
/// ```rust,ignore
/// try_range!(myobj.mylist[0 => 4], &client);
/// ```
#[macro_export]
macro_rules! try_range {
    ($obj: ident.$prop: ident[$start:expr => $end:expr], $conn: expr) => {{
        $obj.$prop.try_range(stringify!($prop), &$obj, $start, $end, &$conn)
    }}
}

/// Creates an iterable of all elements in `$obj.$prop`.
/// The property must be a List.
#[macro_export]
macro_rules! try_iter {
    ($obj: ident.$prop: ident, $conn: expr) => {{
        $obj.$prop.try_iter(stringify!($prop), &$obj, &$conn)
    }}
}

/// Checks if an element is in a List or a Set.
#[macro_export]
macro_rules! contains {
    ($obj: ident.$prop: ident, $el: expr, $conn: expr) => {{
        $obj.$prop.contains(stringify!($prop), &$obj, &$el, &$conn)
    }}
}

/// Removes occurences of an element in a List or a Set.
#[macro_export]
macro_rules! remove {
    ($obj: ident.$prop: ident, $el: expr, $conn: expr) => {{
        $obj.$prop.remove(stringify!($prop), &$obj, &$el, &$conn)
    }}
}

/// Find an element by a unique index.
///
/// # Examples
///
/// ```rust
/// # #[macro_use(model, create)] extern crate ohmers;
/// # extern crate rustc_serialize;
/// # extern crate redis;
/// # use ohmers::Ohmer;
/// # use redis::Commands;
/// model!(
///     OperativeSystem {
///         uniques {
///             name:String = "".to_string();
///         };
///         major_version:u8 = 0;
///         minor_version:u8 = 0;
///     });
/// # fn main() {
/// # let client = redis::Client::open("redis://127.0.0.1/").unwrap();
/// # let _:bool = client.del("OperativeSystem:uniques:name").unwrap();
/// create!(OperativeSystem { name: "Windows".to_owned(), major_version: 10, }, &client);
/// create!(OperativeSystem { name: "GNU/Linux".to_owned(), major_version: 3, minor_version: 14, }, &client);
/// create!(OperativeSystem { name: "OS X".to_owned(), major_version: 10, minor_version: 10, }, &client);
/// assert_eq!(ohmers::with::<OperativeSystem, _>("name", "OS X", &client).unwrap().unwrap().major_version, 10);
/// # }
/// ```
pub fn with<T: Ohmer, S: ToRedisArgs>(property: &str, value: S, r: &redis::Client) -> Result<Option<T>, DecoderError> {
    let mut obj = T::default();

    let opt_id:Option<usize> = try!(r.hget(format!("{}:uniques:{}", obj.get_class_name(), property), value));

    let id = match opt_id {
        Some(id) => id,
        None => return Ok(None),
    };
    try!(obj.load(id, r));
    Ok(Some(obj))
}

/// Gets an element by id.
///
/// # Examples
///
/// ```rust
/// # #[macro_use(model, create)] extern crate ohmers;
/// # extern crate rustc_serialize;
/// # extern crate redis;
/// # use ohmers::Ohmer;
/// # use redis::Commands;
/// model!(
///     Server {
///         name:String = "".to_string();
///     });
/// # fn main() {
/// # let client = redis::Client::open("redis://127.0.0.1/").unwrap();
/// let server = create!(Server { name: "My Server".to_owned(), }, &client).unwrap();
/// assert_eq!(&*ohmers::get::<Server>(server.id, &client).unwrap().name, "My Server");
/// # }
/// ```
pub fn get<T: Ohmer>(id: usize, r: &redis::Client) -> Result<T, DecoderError> {
    let mut obj = T::default();
    try!(obj.load(id, r));
    Ok(obj)
}

/// Gets a query for all elements.
///
/// # Examples
///
/// ```rust
/// # #[macro_use(model, create, new)] extern crate ohmers;
/// # extern crate rustc_serialize;
/// # extern crate redis;
/// # use ohmers::Ohmer;
/// # use redis::Commands;
/// model!(
///     URL {
///         domain:String = "".to_string();
///         path:String = "/".to_string();
///     });
/// # fn main() {
/// # let client = redis::Client::open("redis://127.0.0.1/").unwrap();
/// # let _:bool = client.del("URL:all").unwrap();
/// # let _:bool = client.del("URL:id").unwrap();
/// create!(URL { domain: "example.com".to_owned(), }, &client).unwrap();
/// create!(URL { domain: "example.org".to_owned(), path: "/ping".to_owned(), }, &client).unwrap();
/// assert_eq!(ohmers::all_query::<URL>(&client).unwrap().sort("path", None, true, true).unwrap().collect::<Vec<_>>(),
///     vec![
///         new!(URL { id: 1, domain: "example.com".to_owned(), }),
///         new!(URL { id: 2, domain: "example.org".to_owned(), path: "/ping".to_owned(), }),
///     ]);
/// # }
/// ```
pub fn all_query<'a, T: 'a + Ohmer>(r: &'a redis::Client) -> Result<Query<'a, T>, OhmerError> {
    let class_name = T::default().get_class_name();
    Ok(Query::<'a, T>::new(stal::Set::Key(format!("{}:all", class_name).as_bytes().to_vec()), r))
}

pub fn all<'a, T: 'a + Ohmer>(r: &'a redis::Client) -> Result<Iter<T>, OhmerError> {
    Ok(try!(try!(all_query(r)).try_iter()))
}

pub trait Ohmer : rustc_serialize::Encodable + rustc_serialize::Decodable + Default + Sized {
    fn id_field(&self) -> String { "id".to_string() }
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);

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

    fn encoder(&self) -> Result<Encoder, OhmerError> {
        let mut encoder = Encoder::new();
        encoder.id_field = self.id_field();
        try!(self.encode(&mut encoder));
        Ok(encoder)
    }

    fn uniques_indices(&self, encoder: &Encoder
            ) -> Result<(HashMap<String, String>, HashMap<String, Vec<String>>), OhmerError> {
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
            } else if key.len() > 3 && &key[key.len() - 3..] == "_id" &&
                index_fields.remove(&key[..key.len() - 3]) {
                indices.insert(key.clone(), vec![encoder.attributes[pos + 1].clone()]);
            }
        }
        if unique_fields.len() > 0 {
            return Err(OhmerError::UnknownIndex(unique_fields.iter().next().unwrap().to_string()));
        }
        Ok((uniques, indices))

    }

    fn save(&mut self, r: &redis::Client) -> Result<(), OhmerError> {
        let encoder = try!(self.encoder());
        let (uniques, indices) = try!(self.uniques_indices(&encoder));
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

    fn delete(self, r: &redis::Client) -> Result<(), OhmerError> {
        let encoder = try!(self.encoder());
        let (uniques, _) = try!(self.uniques_indices(&encoder));

        let mut tracked = encoder.sets;
        tracked.extend(encoder.counters);
        tracked.extend(encoder.lists);

        let mut model = HashMap::new();
        let id = self.id();
        let name = self.get_class_name();
        model.insert("key", format!("{}:{}", name, id));
        model.insert("id", format!("{}", id));
        model.insert("name", name);

        let script = redis::Script::new(DELETE);
        let _:() = try!(script
                .arg(try!(msgpack_encode(&model)))
                .arg(try!(msgpack_encode(&uniques)))
                .arg(try!(msgpack_encode(&tracked)))
                .invoke(&try!(r.get_connection())));
        Ok(())
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct Reference<T: Ohmer> {
    id: usize,
    phantom: PhantomData<T>,
}

impl<T: Ohmer> Reference<T> {
    pub fn new() -> Self {
        Reference { id: 0, phantom: PhantomData }
    }

    pub fn with_value(obj: &T) -> Self {
        Reference { id: obj.id(), phantom: PhantomData }
    }

    pub fn get(&self, r: &redis::Client) -> Result<T, DecoderError> {
        get(self.id, r)
    }

    pub fn set(&mut self, obj: &T) {
        self.id = obj.id();
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct Collection<T: Ohmer> {
    phantom: PhantomData<T>,
}

impl<T: Ohmer> Collection<T> {
    pub fn new() -> Self {
        Collection { phantom: PhantomData }
    }

    pub fn all<'a, P: Ohmer>(&'a self, property: &str, parent: &P, r: &'a redis::Client) -> Query<T> {
        Query::<T>::find(&*format!("{}_id", property.to_ascii_lowercase()), &*format!("{}", parent.id()), r)
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct List<T: Ohmer> {
    phantom: PhantomData<T>,
}

impl<T: Ohmer> List<T> {
    pub fn new() -> Self {
        List { phantom: PhantomData }
    }

    fn key_name<P: Ohmer>(&self, property: &str, parent: &P) -> Result<String, OhmerError> {
        let id = parent.id();
        if id == 0 {
            Err(OhmerError::NotSaved)
        } else {
            Ok(format!("{}:{}:{}", parent.get_class_name(), property, parent.id()))
        }
    }

    pub fn len<P: Ohmer>(&self, property: &str, parent: &P, r: &redis::Client) -> Result<usize, OhmerError> {
        Ok(try!(r.llen(try!(self.key_name(property, parent)))))
    }

    pub fn push_back<P: Ohmer>(&self, property: &str, parent: &P, obj: &T, r: &redis::Client) -> Result<(), OhmerError> {
        Ok(try!(r.rpush(try!(self.key_name(property, parent)), obj.id())))
    }

    pub fn pop_back<P: Ohmer>(&self, property: &str, parent: &P, r: &redis::Client) -> Result<Option<T>, OhmerError> {
        Ok(match try!(r.rpop(try!(self.key_name(property, parent)))) {
            Some(id) => Some(try!(get(id, r))),
            None => None,
        })
    }

    pub fn push_front<P: Ohmer>(&self, property: &str, parent: &P, obj: &T, r: &redis::Client) -> Result<(), OhmerError> {
        Ok(try!(r.lpush(try!(self.key_name(property, parent)), obj.id())))
    }

    pub fn pop_front<P: Ohmer>(&self, property: &str, parent: &P, r: &redis::Client) -> Result<Option<T>, OhmerError> {
        Ok(match try!(r.lpop(try!(self.key_name(property, parent)))) {
            Some(id) => Some(try!(get(id, r))),
            None => None,
        })
    }

    pub fn first<P: Ohmer>(&self, property: &str, parent: &P, r: &redis::Client) -> Result<Option<T>, OhmerError> {
        Ok(match try!(r.lindex(try!(self.key_name(property, parent)), 0)) {
            Some(id) => Some(try!(get(id, r))),
            None => None,
        })
    }

    pub fn last<P: Ohmer>(&self, property: &str, parent: &P, r: &redis::Client) -> Result<Option<T>, OhmerError> {
        Ok(match try!(r.lindex(try!(self.key_name(property, parent)), -1)) {
            Some(id) => Some(try!(get(id, r))),
            None => None,
        })
    }

    pub fn try_range<'a, P: Ohmer>(&'a self, property: &str, parent: &P, start: isize, end: isize, r: &'a redis::Client) -> Result<Iter<T>, OhmerError> {
        let ids:Vec<usize> = try!(r.lrange(try!(self.key_name(property, parent)), start, end));
        Ok(Iter::new(ids.into_iter(), r))
    }

    pub fn try_iter<'a, P: Ohmer>(&'a self, property: &str, parent: &P, r: &'a redis::Client) -> Result<Iter<T>, OhmerError> {
        self.try_range(property, parent, 0, -1, r)
    }

    pub fn contains<P: Ohmer>(&self, property: &str, parent: &P, obj: &T, r: &redis::Client) -> Result<bool, OhmerError> {
        let ids:Vec<usize> = try!(r.lrange(try!(self.key_name(property, parent)), 0, -1));
        Ok(ids.contains(&obj.id()))
    }

    pub fn remove<P: Ohmer>(&self, property: &str, parent: &P, obj: &T, r: &redis::Client) -> Result<usize, OhmerError> {
        Ok(try!(r.lrem(try!(self.key_name(property, parent)), 0, obj.id())))
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
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
    DecoderError,
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

impl From<DecoderError> for OhmerError {
    fn from(_: DecoderError) -> OhmerError {
        OhmerError::DecoderError
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
        $obj.$prop.incr(&$obj, stringify!($prop), 1, $client)
    }}
}

#[macro_export]
macro_rules! decr {
    ($obj: expr, $prop: ident, $client: expr) => {{
        $obj.$prop.incr(&$obj, stringify!($prop), -1, $client)
    }}
}

pub struct Query<'a, T: 'a + Ohmer> {
    set: stal::Set,
    r: &'a redis::Client,
    phantom: PhantomData<T>,
}

impl<'a, T: Ohmer> Query<'a, T> {
    pub fn new(set: stal::Set, r: &'a redis::Client) -> Self {
        Query { set: set, phantom: PhantomData, r: r }
    }

    pub fn from_keys(kv: &[(&str, &str)], r: &'a redis::Client) -> Self {
        let set = stal::Set::Inter(kv.iter().map(|kv| Query::<T>::key(kv.0, kv.1)).collect());
        Query::new(set, r)
    }

    pub fn key(field: &str, value: &str) -> stal::Set {
        stal::Set::Key(T::default().key_for_index(field, value).as_bytes().to_vec())
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
        Iter::from_ops(self.set.ids().solve(), self.r)
    }

    pub fn try_into_iter(self) -> Result<Iter<'a, T>, OhmerError> {
        Iter::from_ops(self.set.into_ids().solve(), self.r)
    }

    pub fn sort(&self, by: &str, limit: Option<(usize, usize)>, asc: bool, alpha: bool) -> Result<Iter<'a, T>, OhmerError> {
        let default = T::default();
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
        Iter::from_ops(stal.solve(), self.r)
    }
}

pub struct Iter<'a, T> {
    r: &'a redis::Client,
    iter: std::vec::IntoIter<usize>,
    phantom: PhantomData<T>,
}

impl<'a, T: Ohmer> Iter<'a, T> {
    fn new(iter: std::vec::IntoIter<usize>, r: &'a redis::Client) -> Self {
        Iter {
            iter: iter,
            r: r,
            phantom: PhantomData,
        }
    }

    fn from_ops(ops: (Vec<Vec<Vec<u8>>>, usize), r: &'a redis::Client) -> Result<Self, OhmerError> {
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
                Err(_) => None,
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len(), Some(self.iter.len()))
    }
}
