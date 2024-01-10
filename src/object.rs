use std::{
    fmt::{self, Debug, Formatter},
};

use crate::encode::{sint_to_bytes, uint_to_bytes};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ValueClass {
    String,
    Bytes,
    Collection,
    Fixed,
}

impl From<u8> for ValueClass {
    fn from(value: u8) -> Self {
        match value {
            0 => ValueClass::String,
            1 => ValueClass::Bytes,
            2 => ValueClass::Collection,
            3 => ValueClass::Fixed,
            _ => panic!("Invalid object class"),
        }
    }
}

#[derive(Clone)]
pub enum Value {
    String(String),
    Bytes(Vec<u8>),

    Map(Vec<(Object, Object)>),
    List(Vec<Object>),

    Bool(bool),
    SInt(i128),
    UInt(u128),
    Float32(f32),
    Float64(f64),
    Null,
    Timestamp32(u32),

    UserDefined { id: u8, data: Vec<u8> },
}

impl Value {
    pub fn class(&self) -> u8 {
        match self {
            Value::String(_) => 0,
            Value::Bytes(_) => 1,
            Value::Map(_) | Value::List(_) => 2,
            Value::Bool(_)
            | Value::SInt(_)
            | Value::UInt(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::Null
            | Value::Timestamp32(_)
            | Value::UserDefined { id: _, data: _ } => 3,
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::UserDefined { id, data } => {
                write!(f, "UserDefined {{ id: {}, data: {:?} }}", id, data)
            }
            Value::String(v) => v.fmt(f),
            Value::Bytes(v) => v.fmt(f),
            Value::Map(v) => v.fmt(f),
            Value::List(v) => v.fmt(f),
            Value::Bool(v) => v.fmt(f),
            Value::SInt(v) => v.fmt(f),
            Value::UInt(v) => v.fmt(f),
            Value::Float32(v) => v.fmt(f),
            Value::Float64(v) => v.fmt(f),
            Value::Timestamp32(v) => v.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct Object {
    pub value: Value,
    pub length: usize,
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Value::Map(m) = &self.value {
            write!(f, "{{")?;
            
            for (i, (k, v)) in m.iter().enumerate() {
                write!(f, "{:?}:{:?}", k, v)?;
                if i != m.len() - 1 {
                    write!(f, ", ")?;
                }
            }

            write!(f, "}}")
        } else {
            self.value.fmt(f)
        }

        // f.debug_struct("Object").field("value", &self.value).field("length", &self.length).finish()
    }
}

impl Object {
    pub fn class(&self) -> u8 {
        self.value.class()
    }

    pub fn string(s: String) -> Self {
        Object {
            length: s.len(),
            value: Value::String(s),
        }
    }

    pub fn bytes(b: Vec<u8>) -> Self {
        Object {
            length: b.len(),
            value: Value::Bytes(b),
        }
    }

    pub fn map(m: Vec<(Object, Object)>) -> Self {
        Object {
            length: m.len(),
            value: Value::Map(m),
        }
    }

    pub fn list(l: Vec<Object>) -> Self {
        Object {
            length: l.len(),
            value: Value::List(l),
        }
    }

    pub fn bool(b: bool) -> Self {
        Object {
            length: 0,
            value: Value::Bool(b),
        }
    }

    pub fn sint(i: i128) -> Self {
        Object {
            length: sint_to_bytes(i).len(),
            value: Value::SInt(i),
        }
    }

    pub fn uint(u: u128) -> Self {
        Object {
            length: uint_to_bytes(u).len(),
            value: Value::UInt(u),
        }
    }

    pub fn float32(f: f32) -> Self {
        Object {
            length: 4,
            value: Value::Float32(f),
        }
    }

    pub fn float64(f: f64) -> Self {
        Object {
            length: 8,
            value: Value::Float64(f),
        }
    }

    pub fn null() -> Self {
        Object {
            length: 0,
            value: Value::Null,
        }
    }

    #[allow(dead_code)]
    pub fn timestamp32(t: u32) -> Self {
        Object {
            length: 4,
            value: Value::Timestamp32(t),
        }
    }

    /*
       class: the ValueClass decoded from the TYPE section
       length: the length of the object, decoded from the LENGTH section
    */
    pub fn from_class(class: ValueClass, length: usize) -> Self {
        let mut length = length;

        let value: Value = match class {
            ValueClass::String => Value::String(String::new()),
            ValueClass::Bytes => Value::Bytes(Vec::new()),
            ValueClass::Collection => {
                // check lower bit of length
                if length & 1 == 1 {
                    length = length >> 1;
                    Value::List(Vec::with_capacity(length))
                } else {
                    length = length >> 1;
                    Value::Map(Vec::with_capacity(length))
                }
            }
            ValueClass::Fixed => match length {
                0..=16 => Value::SInt(0),
                17..=32 => {
                    length -= 16;
                    Value::UInt(0)
                }
                33 => {
                    length = 4;
                    Value::Float32(0.0)
                }
                34 => {
                    length = 8;
                    Value::Float64(0.0)
                }
                35 => {
                    length = 0;
                    Value::Null
                }
                36 => {
                    length = 0;
                    Value::Bool(false)
                }
                37 => {
                    length = 0;
                    Value::Bool(true)
                }
                38 => {
                    length = 4;
                    Value::Timestamp32(0)
                }
                id => Value::UserDefined {
                    id: id as u8,
                    data: Vec::new(),
                },
            },
        };

        Object { value, length }
    }
}

impl From<String> for Object {
    fn from(s: String) -> Self {
        Object::string(s)
    }
}

impl From<&str> for Object {
    fn from(s: &str) -> Self {
        Object::string(s.to_string())
    }
}

impl From<Vec<u8>> for Object {
    fn from(b: Vec<u8>) -> Self {
        Object::bytes(b)
    }
}

impl From<Vec<(Object, Object)>> for Object {
    fn from(m: Vec<(Object, Object)>) -> Self {
        Object::map(m)
    }
}

impl From<Vec<Object>> for Object {
    fn from(l: Vec<Object>) -> Self {
        Object::list(l)
    }
}

impl From<i8> for Object {
    fn from(i: i8) -> Self {
        Object::sint(i as i128)
    }
}

impl From<u8> for Object {
    fn from(u: u8) -> Self {
        Object::uint(u as u128)
    }
}

impl From<i16> for Object {
    fn from(i: i16) -> Self {
        Object::sint(i as i128)
    }
}

impl From<u16> for Object {
    fn from(u: u16) -> Self {
        Object::uint(u as u128)
    }
}

impl From<i32> for Object {
    fn from(i: i32) -> Self {
        Object::sint(i as i128)
    }
}

impl From<u32> for Object {
    fn from(u: u32) -> Self {
        Object::uint(u as u128)
    }
}

impl From<i64> for Object {
    fn from(i: i64) -> Self {
        Object::sint(i as i128)
    }
}

impl From<u64> for Object {
    fn from(u: u64) -> Self {
        Object::uint(u as u128)
    }
}

impl From<i128> for Object {
    fn from(i: i128) -> Self {
        Object::sint(i)
    }
}

impl From<u128> for Object {
    fn from(u: u128) -> Self {
        Object::uint(u)
    }
}

impl From<f32> for Object {
    fn from(f: f32) -> Self {
        Object::float32(f)
    }
}

impl From<f64> for Object {
    fn from(f: f64) -> Self {
        Object::float64(f)
    }
}

impl From<bool> for Object {
    fn from(b: bool) -> Self {
        Object::bool(b)
    }
}