use std::io::{self, Write};

use crate::object::{self, Object, Value};

pub fn headpack_encode(root: Object) -> Vec<u8> {
    let mut buf = Vec::new();

    let mut objects = Vec::new();

    if let object::Value::Map(items) = root.value {
        // flatten the inside of the map
        expand_collections(&mut items.into_iter().map(|(k, v)| vec![k, v]).flatten(), &mut objects);
        // note: the map itself is not included in the objects list because by
        // default if a message has an even number of top-level objects, it is a map
    }
    else if let object::Value::List(elements) = root.value {
        // place a  list marker at the beginning of the objects buffer, so that
        // it consumes every following object and thus creates an odd number of
        // top-level objects (1), which indicates the root is not a map
        objects.push(
            Object { value: Value::List(Vec::new()), length: root.length }
        );
        // flatten the inside of the list
        expand_collections(&mut elements.into_iter(), &mut objects);
    }
    else {
        // root is a single non-collection object
        objects.push(root);
    }

    write_classes_section(&objects, &mut buf);
    write_lengths_section(&objects, &mut buf);

    write_data(objects.into_iter(), &mut buf).unwrap();

    buf
}

pub fn expand_collections(iter: &mut impl Iterator<Item = Object>, into: &mut Vec<Object>) {
    for object in iter {
        match object.value {
            object::Value::List(elements) => {
                if elements.len() % 2 == 0{
                    // push an empty list object
                    into.push(Object {
                        value: object::Value::List(Vec::new()),
                        length: object.length,
                    });
                } // not needed if odd, because it can't be a map

                expand_collections(&mut elements.into_iter(), into);
            }
            object::Value::Map(items) => {
                // push an empty map object
                into.push(Object {
                    value: object::Value::Map(Vec::new()),
                    length: object.length,
                });

                for (k, v) in items.into_iter() {
                    expand_collections(&mut [k, v].into_iter(), into);
                }
            }
            _ => {
                into.push(object);
            }
        }
    }
}

fn write_classes_section(o: &[Object], data: &mut Vec<u8>) {
    let mut len = o.len();

    if len == 3 {
        data.push(classes_join(3, o[0].class(), o[1].class(), o[2].class()));
        return;
    }

    if len == 1 {
        data.push(classes_join(1, o[0].class(), 0, 0));
        return;
    }

    if len == 0 {
        data.push(classes_join(0, 0, 0, 0));
        return;
    }

    len -= 2;
    data.push(classes_join(
        2,
        o[0].class(),
        o[1].class(),
        if (len) >= 3 { 3 } else { len } as u8,
    ));

    let mut i = 2;
    loop {
        match len {
            0 => break,
            1 => {
                data.push(classes_join(o[i].class(), 0, 0, 0));
                break;
            }
            2 => {
                data.push(classes_join(o[i].class(), o[i + 1].class(), 0, 0));
                break;
            }
            _ => {
                // SAFETY: l is guaranteed to be at least 3, so no overflow here
                let minus_3 = len - 3;

                data.push(classes_join(
                    o[i].class(),
                    o[i + 1].class(),
                    o[i + 2].class(),
                    if (minus_3) >= 3 { 3 } else { minus_3 } as u8,
                ));

                len = minus_3;
            }
        }

        i += 3;
    }
}

fn split_into_3_bit_chunks(n: usize) -> Vec<u8> {
    let bits_required_to_store_n = if n > 0 { n.ilog2() } else { 0 } + 1;

    let mut output = Vec::new();

    for i in (0..bits_required_to_store_n).step_by(3) {
        let chunk = (n >> i) & 0b111;
        output.push(chunk as u8);
    }

    output.reverse();
    output
}

pub fn uint_to_bytes(n: u128) -> Vec<u8> {
    let v: Vec<u8> = n
        .to_be_bytes()
        .into_iter()
        .skip_while(|x| *x == 0)
        .collect();

    // if v.is_empty() {
    //     v.push(0);
    // }

    v
}

pub fn uint_from_bytes(bytes: impl Iterator<Item = u8>) -> u128 {
    let mut n = 0;

    for byte in bytes {
        n <<= 8;
        n |= byte as u128;
    }

    n
}

pub fn sint_to_bytes(n: i128) -> Vec<u8> {
    let mut uint_n = (n.unsigned_abs()) << 1;

    if n < 0 {
        // set last bit to 1 to indicate negative
        uint_n |= 1;
    } else {
        // set last bit to 0 to indicate positive
        uint_n &= !1;
    }

    uint_to_bytes(uint_n)
}

pub fn sint_from_bytes(bytes: impl Iterator<Item = u8>) -> i128 {
    let u = uint_from_bytes(bytes) as u128;

    // convert least significant bit to sign
    if u & 1 == 1 {
        let s = -((u >> 1) as i128);
        return if s == 0 { i128::MIN } else { s };
    } else {
        return (u >> 1) as i128;
    }
}

fn write_lengths_section(objects: &[Object], data: &mut Vec<u8>) {
    let mut chunks = Vec::with_capacity(objects.len());

    // convert &[Object] to &[&Object]
    let objects: Vec<&Object> = objects.iter().collect();
    write_length_chunks(&objects, &mut chunks);

    // pair up two four-bit chunks into a byte
    let mut i = 0;
    while i < chunks.len() {
        let mut byte = chunks[i] << 4;
        i += 1;

        if i < chunks.len() {
            byte |= chunks[i];
            i += 1;
        }

        data.push(byte);
    }
}

fn write_length_chunks(objects: &[&Object], chunks: &mut Vec<u8>) {
    for object in objects {
        let length = match &object.value {
            // uint has a variable length but offset by 16
            Value::UInt(_) => object.length + 16,
            // special fixed-length objects
            Value::Float32(_) => 33,
            Value::Float64(_) => 34,
            Value::Null => 35,
            Value::Bool(b) => {
                if *b {
                    37
                } else {
                    36
                }
            }
            Value::Timestamp32(_) => 38,
            Value::UserDefined { id, data: _ } => *id as usize,
            // variable-length objects
            Value::Map(_) => {
                object.length << 1 // set "is list" bit to 0
            }
            Value::List(_) => {
                object.length << 1 | 1 // set "is list" bit to 1
            }
            _ => object.length,
        };

        for triplet in split_into_3_bit_chunks(length) {
            let chunk = (triplet << 1) | 1;
            chunks.push(chunk);
        }

        let last_idx = chunks.len() - 1; // wah wah "borrowed twice"
                                         // set last bit to 0 to indicate end of length chunks
        chunks[last_idx] &= 0b11111110;
    }
}

fn write_data(objects: impl Iterator<Item = Object>, buf: &mut Vec<u8>) -> io::Result<()> {
    for object in objects {
        match object.value {
            Value::String(s) => {
                buf.write(s.as_bytes())?;
            }
            Value::Bytes(b) => {
                buf.write(&b)?;
            }
            Value::SInt(i) => {
                buf.write(&sint_to_bytes(i))?;
            }
            Value::UInt(i) => {
                buf.write(&uint_to_bytes(i))?;
            }
            Value::Float32(f) => {
                buf.write(&f.to_be_bytes())?;
            }
            Value::Float64(f) => {
                buf.write(&f.to_be_bytes())?;
            }
            Value::Timestamp32(t) => {
                buf.write(&t.to_be_bytes())?;
            }
            Value::UserDefined { id: _, data } => {
                buf.write(&data)?;
            }
            // others need no data
            _ => {}
        }
    }

    Ok(())
}

fn classes_join(a: u8, b: u8, c: u8, d: u8) -> u8 {
    // join 4 2-bit values into a byte
    ((a & 0b11) << 6) | ((b & 0b11) << 4) | ((c & 0b11) << 2) | (d & 0b11)
}
