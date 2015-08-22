use std::collections::HashMap;

use msgpack;
use redis;
use rustc_serialize;

#[derive(Debug, Clone)]
pub struct Encoder {
    pub id: usize,
    pub id_field: String,
    pub features: HashMap<String, String>,
    pub attributes: Vec<String>,
}

impl Encoder {
    pub fn new() -> Self {
        Encoder {
            id: 0,
            id_field: "".to_string(),
            features: HashMap::new(),
            attributes: vec![],
        }
    }
}

#[derive(Debug)]
pub enum EncoderError {
    NotImplementedYet,
    RedisError(redis::RedisError),
    MsgPackError(msgpack::encode::serialize::Error),
}

impl From<redis::RedisError> for EncoderError {
    fn from(e: redis::RedisError) -> EncoderError {
        EncoderError::RedisError(e)
    }
}

impl From<msgpack::encode::serialize::Error> for EncoderError {
    fn from(e: msgpack::encode::serialize::Error) -> EncoderError {
        EncoderError::MsgPackError(e)
    }
}

pub type EncodeResult<T> = Result<T, EncoderError>;

macro_rules! emit_fmt {
    ($enc: ident, $e: expr) => {{
        let s = format!("{}", $e);
        let len = $enc.attributes.len();
        if len == 0 || $enc.attributes[len - 1] == "id" {
            let value = s.clone();
            if &*value != "0" {
                $enc.features.insert("id".to_string(), value);
            }
            $enc.attributes.pop();
        } else {
            $enc.attributes.push(s);
        }
        Ok(())
    }}
}

impl rustc_serialize::Encoder for Encoder {
    type Error = EncoderError;

    fn emit_nil(&mut self) -> EncodeResult<()> {
        self.attributes.pop();
        Ok(())
    }

    fn emit_usize(&mut self, v: usize) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_u64(&mut self, v: u64) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_u32(&mut self, v: u32) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_u16(&mut self, v: u16) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_u8(&mut self, v: u8) -> EncodeResult<()> { emit_fmt!(self, v) }

    fn emit_isize(&mut self, v: isize) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_i64(&mut self, v: i64) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_i32(&mut self, v: i32) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_i16(&mut self, v: i16) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_i8(&mut self, v: i8) -> EncodeResult<()> { emit_fmt!(self, v) }

    fn emit_bool(&mut self, v: bool) -> EncodeResult<()> { emit_fmt!(self, if v { 1 } else { 0 }) }

    fn emit_f64(&mut self, v: f64) -> EncodeResult<()> { emit_fmt!(self, v) }
    fn emit_f32(&mut self, v: f32) -> EncodeResult<()> { emit_fmt!(self, v) }

    fn emit_char(&mut self, v: char) -> EncodeResult<()> { emit_fmt!(self, v) }

    fn emit_str(&mut self, v: &str) -> EncodeResult<()> { emit_fmt!(self, v) }

    fn emit_enum<F>(&mut self, _: &str, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_enum_variant<F>(&mut self,
        _: &str,
        _: usize,
        _: usize,
        _: F)
        -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_enum_variant_arg<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   _: &str,
                                   _: usize,
                                   _: usize,
                                   _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         _: usize,
                                         _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }


    fn emit_struct<F>(&mut self, name: &str, _: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        self.features.insert("name".to_string(), name.to_string());
        f(self)
    }

    fn emit_struct_field<F>(&mut self, name: &str, _: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        self.attributes.push(name.to_string());
        f(self)
    }

    fn emit_tuple<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_tuple_arg<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_tuple_struct<F>(&mut self, _: &str, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }
    fn emit_tuple_struct_arg<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_option<F>(&mut self, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_option_none(&mut self) -> EncodeResult<()> {
        self.emit_nil()
    }

    fn emit_option_some<F>(&mut self, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        f(self)
    }

    fn emit_seq<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_seq_elt<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_map<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_map_elt_key<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }

    fn emit_map_elt_val<F>(&mut self, _: usize, _: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder) -> EncodeResult<()>,
    {
        Err(EncoderError::NotImplementedYet)
    }
}

pub fn msgpack_encode<T: rustc_serialize::Encodable>(t: &T) -> Result<Vec<u8>, msgpack::encode::serialize::Error> {
    let mut buf = Vec::new();
    try!(t.encode(&mut msgpack::Encoder::new(&mut buf)));
    Ok(buf)
}
