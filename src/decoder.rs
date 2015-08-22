use std::ascii::AsciiExt;
use std::collections::HashMap;

use redis;
use rustc_serialize;

#[derive(Debug, Clone, PartialEq)]
enum DecoderStatus {
    Unnamed,
    Normal,
    Reference,
}

#[derive(Debug)]
pub enum DecoderError {
    NotImplementedYet,
    ExpectedError(String, String),
    RedisError(redis::RedisError),
    ApplicationError(String),
}

impl From<redis::RedisError> for DecoderError {
    fn from(e: redis::RedisError) -> DecoderError {
        DecoderError::RedisError(e)
    }
}

type DecodeResult<T> = Result<T, DecoderError>;

pub struct Decoder {
    properties: HashMap<String, String>,
    stack: Vec<Option<String>>,
    status: DecoderStatus,
}

impl Decoder {
    pub fn new(properties: HashMap<String, String>) -> Decoder {
        Decoder {
            properties: properties,
            stack: vec![],
            status: DecoderStatus::Unnamed,
        }
    }
}

macro_rules! read_primitive {
    ($name:ident, $ty:ident) => {
        fn $name(&mut self) -> DecodeResult<$ty> {
            match self.stack.pop() {
                Some(opt_s) => match opt_s {
                    Some(s) => match s.parse() {
                        Ok(v) => Ok(v),
                        Err(_) => Err(DecoderError::ExpectedError("Number".to_string(), s)),
                    },
                    None => Err(DecoderError::ExpectedError("Number".to_string(), "None".to_string()))
                },
                None => Err(DecoderError::ExpectedError("Number".to_string(), "Not found".to_string()))
            }
        }
    }
}

impl rustc_serialize::Decoder for Decoder {
    type Error = DecoderError;

    fn read_nil(&mut self) -> DecodeResult<()> {
        Ok(())
    }

    fn read_usize(&mut self) -> DecodeResult<usize> {
        let v = match self.stack.pop() {
            Some(opt_s) => match opt_s {
                Some(s) => match s.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(DecoderError::ExpectedError("Number".to_string(), s)),
                },
                None => return Err(DecoderError::ExpectedError("Number".to_string(), "None".to_string()))
            },
            None => return Err(DecoderError::ExpectedError("Number".to_string(), "Not found".to_string()))
        };
        self.status = DecoderStatus::Normal;
        Ok(v)
    }

    read_primitive! { read_u8, u8 }
    read_primitive! { read_u16, u16 }
    read_primitive! { read_u32, u32 }
    read_primitive! { read_u64, u64 }
    read_primitive! { read_isize, isize }
    read_primitive! { read_i8, i8 }
    read_primitive! { read_i16, i16 }
    read_primitive! { read_i32, i32 }
    read_primitive! { read_i64, i64 }

    fn read_f32(&mut self) -> DecodeResult<f32> {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_f64(&mut self) -> DecodeResult<f64> {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_char(&mut self) -> DecodeResult<char> {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_str(&mut self) -> DecodeResult<String> {
        match self.stack.pop() {
            Some(opt_s) => match opt_s {
                Some(s) => Ok(s),
                None => Err(DecoderError::ExpectedError("String".to_string(), "None".to_string()))
            },
            None => Err(DecoderError::ExpectedError("String".to_string(), "Not found".to_string()))
        }
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, _names: &[&str],
                               mut _f: F) -> DecodeResult<T>
        where F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T, F>(&mut self,
                                         _name: &str,
                                         idx: usize,
                                         f: F)
                                         -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T, F>(&mut self, _name: &str, _len: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_struct_field<T, F>(&mut self,
                               name: &str,
                               _idx: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        if self.status != DecoderStatus::Reference {
            match self.properties.remove(name) {
                Some(v) => self.stack.push(Some(v)),
                None => {
                    match self.properties.remove(&*format!("{}_id", name).to_ascii_lowercase()) {
                        Some(id) => {
                            self.status = DecoderStatus::Reference;
                            self.stack.push(Some(id));
                        },
                        None => {
                            self.stack.push(None);
                        }
                    }
                }
            }
        }
        f(self)
    }

    fn read_tuple<T, F>(&mut self, _tuple_len: usize, _f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_tuple_arg<T, F>(&mut self, idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T, F>(&mut self,
                               _name: &str,
                               len: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple(len, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self,
                                   idx: usize,
                                   f: F)
                                   -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T, F>(&mut self, mut f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, bool) -> DecodeResult<T>,
    {
        let opt = match self.stack.last() {
            Some(ref el) => el.is_some(),
            None => return Err(DecoderError::ExpectedError("Option".to_string(), "Not found".to_string())),
        };
        f(self, opt)
    }

    fn read_seq<T, F>(&mut self, _f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, _f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        Err(DecoderError::NotImplementedYet)
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> DecoderError {
        DecoderError::ApplicationError(err.to_string())
    }
}
