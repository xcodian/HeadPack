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
            },
            serde_json::Value::String(s) => {
                Self::string(s)
            },
            serde_json::Value::Array(elements) => {
                let mut array = Vec::with_capacity(elements.len());
                
                for element in elements {
                    array.push(Self::from_json(element));
                }
                
                Object::list(array)
            },
            serde_json::Value::Object(map) => {
                let mut pairs = Vec::with_capacity(map.len());

                for (key, value) in map {
                    pairs.push((Self::string(key), Self::from_json(value)));
                }

                Object::map(pairs)
            },
        }
    }
}