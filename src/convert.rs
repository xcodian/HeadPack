use base64::Engine;
use serde_json::{json, Number};

use crate::object;
use crate::object::Object;

impl Object {
    pub fn from_json(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Self::null(),
            serde_json::Value::Bool(b) => Self::bool(b),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    Self::sint(n.as_i64().unwrap() as i128)
                } else if n.is_u64() {
                    Self::uint(n.as_u64().unwrap() as u128)
                } else if n.is_f64() {
                    let f = n.as_f64().unwrap();

                    // check if it fits in f32
                    if f as f32 as f64 == f {
                        Self::float32(f as f32)
                    } else {
                        Self::float64(f)
                    }
                } else {
                    unreachable!()
                }
            }
            serde_json::Value::String(s) => Self::string(s),
            serde_json::Value::Array(elements) => {
                let mut array = Vec::with_capacity(elements.len());

                for element in elements {
                    array.push(Self::from_json(element));
                }

                Object::list(array)
            }
            serde_json::Value::Object(map) => {
                let mut pairs = Vec::with_capacity(map.len());

                for (key, value) in map {
                    pairs.push((Self::string(key), Self::from_json(value)));
                }

                Object::map(pairs)
            }
        }
    }

    pub fn into_json(self) -> serde_json::Value {
        match self.value {
            object::Value::String(s) => serde_json::Value::String(s),
            object::Value::Bytes(b) => {
                serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(&b))
            }
            object::Value::Map(m) => {
                let mut json_map = serde_json::Map::with_capacity(m.len());

                for (key, value) in m {
                    match key.value {
                        object::Value::String(k) => {
                            json_map.insert(k, value.into_json());
                        }
                        _ => {
                            panic!("Map keys must be strings");
                        }
                    }
                }

                serde_json::Value::Object(json_map)
            }
            object::Value::List(l) => {
                let mut json_list = Vec::with_capacity(l.len());

                for element in l {
                    json_list.push(element.into_json());
                }

                serde_json::Value::Array(json_list)
            }
            object::Value::Bool(b) => serde_json::Value::Bool(b),
            object::Value::SInt(i) => {
                if i >= i64::MIN as i128 && i <= i64::MAX as i128 {
                    serde_json::Value::Number(Number::from(i as i64))
                } else {
                    serde_json::Value::String(i.to_string())
                }
            }
            object::Value::UInt(i) => {
                if i <= u64::MAX as u128 {
                    serde_json::Value::Number(Number::from(i as u64))
                } else {
                    serde_json::Value::String(i.to_string())
                }
            }
            object::Value::Float32(f) => {
                let maybe_n = Number::from_f64(f as f64);

                match maybe_n {
                    Some(n) => serde_json::Value::Number(n),
                    None => serde_json::Value::String(f.to_string()),
                }
            }
            object::Value::Float64(f) => {
                let maybe_n = Number::from_f64(f);

                match maybe_n {
                    Some(n) => serde_json::Value::Number(n),
                    None => serde_json::Value::String(f.to_string()),
                }
            }
            object::Value::Null => serde_json::Value::Null,
            object::Value::Timestamp32(t) => serde_json::Value::Number(t.into()),
            object::Value::UserDefined { id, data } => {
                json!({
                    "id": id,
                    "data": base64::engine::general_purpose::STANDARD.encode(&data),
                })
            }
        }
    }
}
