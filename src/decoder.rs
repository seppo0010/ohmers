use std::collections::HashMap;

use redis;
use rustc_serialize;

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
}

impl Decoder {
    pub fn new(properties: HashMap<String, String>) -> Decoder {
        Decoder {
            properties: properties,
            stack: vec![],
        }
    }
}

impl rustc_serialize::Decoder for Decoder {
    type Error = DecoderError;

    fn read_nil(&mut self) -> DecodeResult<()> {
        Ok(())
    }

    fn read_usize(&mut self) -> DecodeResult<usize> {
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

    fn read_u8(&mut self) -> DecodeResult<u8> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_u16(&mut self) -> DecodeResult<u16> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_u32(&mut self) -> DecodeResult<u32> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_u64(&mut self) -> DecodeResult<u64> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_isize(&mut self) -> DecodeResult<isize> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_i8(&mut self) -> DecodeResult<i8> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_i16(&mut self) -> DecodeResult<i16> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_i32(&mut self) -> DecodeResult<i32> {
        Err(DecoderError::NotImplementedYet)
    }
    fn read_i64(&mut self) -> DecodeResult<i64> {
        Err(DecoderError::NotImplementedYet)
    }

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
        self.stack.push(self.properties.remove(name));
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
