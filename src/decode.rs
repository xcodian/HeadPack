use std::collections::VecDeque;

use crate::{
    encode::{sint_from_bytes, uint_from_bytes},
    object::{Object, Value, ValueClass},
};

/// warning: mutates `data`
pub fn headpack_message_decode(mut data: VecDeque<u8>) -> Object {
    let kinds = decode_classes_section(&mut data);
    let objects = decode_lengths_section(&kinds, &mut data);

    let mut collapsed = Vec::with_capacity(objects.len());

    collapse_collections(&mut objects.into_iter(), &mut collapsed, -1);

    if collapsed.len() == 1 {
        return collapsed.pop().unwrap();
    }

    if collapsed.len() % 2 == 0 {
        let mut map_items = Vec::with_capacity(collapsed.len() / 2);

        while collapsed.len() >= 2 {
            let value = collapsed.pop().unwrap();
            let key = collapsed.pop().unwrap();

            map_items.push((key, value))
        }

        map_items.reverse();
        Object::map(map_items)
    } else {
        Object::list(collapsed)
    }
}

fn classes_split(byte: u8) -> (u8, u8, u8, u8) {
    // split a byte up into 4 2-bit values
    let a = (byte >> 6) & 0b11;
    let b = (byte >> 4) & 0b11;
    let c = (byte >> 2) & 0b11;
    let d = byte & 0b11;

    (a, b, c, d)
}

fn lengths_split(byte: u8) -> (u8, u8, u8, u8) {
    // split a byte up into a 3-bit value, a 1-bit value, then a 3-bit value and a 1-bit value
    let a = (byte >> 5) & 0b111;
    let b = (byte >> 4) & 0b1;
    let c = (byte >> 1) & 0b111;
    let d = byte & 0b1;

    (a, b, c, d)
}

fn decode_classes_section(data: &mut VecDeque<u8>) -> Vec<ValueClass> {
    let (first_len, val2, val3, val4) = classes_split(data.pop_front().unwrap());

    if first_len == 0 {
        return vec![];
    }

    if first_len == 3 {
        // all 3 values are types, and this is it
        return vec![val2.into(), val3.into(), val4.into()];
    }

    assert_eq!(first_len, 2);

    let mut classes = vec![val2.into(), val3.into()];
    let mut next_len = val4;

    while next_len > 0 {
        let (val1, val2, val3, val4) = classes_split(data.pop_front().unwrap());

        classes.push(val1.into());

        if next_len >= 2 {
            classes.push(val2.into());
        }

        if next_len == 3 {
            classes.push(val3.into());
            next_len = val4;
        } else {
            break;
        }
    }

    classes
}

fn decode_lengths_section(classes: &[ValueClass], buf: &mut VecDeque<u8>) -> Vec<Object> {
    // chunks of (3-byte size, continue flag)
    let mut chunks: VecDeque<(u8, bool)> = VecDeque::with_capacity(classes.len());

    // output array
    let mut objects = Vec::with_capacity(classes.len());

    // current index into `kinds`
    let mut idx = 0;

    // current length of the object we're decoding
    let mut length = 0;

    while idx < classes.len() {
        let (length_chunk, continue_flag) = lengths_next_chunk(&mut chunks, buf);

        // combine the length chunk with the current length
        length = (length << 3) | length_chunk as usize;

        // this length doesn't fit in the current amount of chunks
        if continue_flag {
            continue;
        }

        // we're done with this length
        objects.push(Object::from_class(classes[idx], length));

        idx += 1;
        length = 0;
    }

    // copy data into string and bytes objects
    for object in objects.iter_mut() {
        match object.value {
            Value::String(ref mut string) => {
                // this does no validation, but it's not supposed to (i think?)
                let chars = buf.drain(..object.length).map(|b| b as char);
                string.extend(chars);
            }
            Value::Bytes(ref mut bytes) => {
                bytes.extend(buf.drain(..object.length));
            }
            Value::SInt(ref mut i) => {
                *i = sint_from_bytes(buf.drain(..object.length));
            }
            Value::UInt(ref mut u) => {
                *u = uint_from_bytes(buf.drain(..object.length));
            }
            Value::Float32(ref mut f) => {
                let arr = buf
                    .drain(..object.length)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .expect("invalid length for Float32");

                *f = f32::from_be_bytes(arr);
            }
            Value::Float64(ref mut f) => {
                let arr = buf
                    .drain(..object.length)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .expect("invalid length for Float64");

                *f = f64::from_be_bytes(arr);
            }
            Value::Timestamp32(ref mut t) => {
                let arr = buf
                    .drain(..object.length)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .expect("invalid length for Timestamp32");

                *t = u32::from_be_bytes(arr);
            }
            Value::UserDefined {
                id: _,
                ref mut data,
            } => {
                data.extend(buf.drain(..object.length));
            }
            Value::Map(_) | Value::List(_) | Value::Bool(_) | Value::Null => {}
        };
    }

    objects
}

pub fn collapse_collections(
    iter: &mut impl Iterator<Item = Object>,
    into: &mut Vec<Object>,
    limit: isize,
) {
    let mut taken = 0;

    loop {
        if limit != -1 && taken >= limit {
            break;
        }

        let object = iter.next();
        if object.is_none() && limit == -1 {
            break;
        }
        let mut object = object.expect("not enough objects to satisfy collection");

        match object.value {
            Value::List(ref mut l) => {
                collapse_collections(iter, l, object.length as isize);
            }
            Value::Map(ref mut v) => {
                println!("map!");

                let mut flat = Vec::with_capacity(object.length);
                collapse_collections(iter, &mut flat, (object.length * 2) as isize);
                let mut flat = VecDeque::from(flat);

                while flat.len() >= 2 {
                    let key = flat.pop_front().unwrap();
                    let value = flat.pop_front().unwrap();

                    v.push((key, value))
                }
                
                assert_eq!(flat.len(), 0);
            }
            _ => {}
        }

        into.push(object);
        taken += 1;
    }
}

fn lengths_next_chunk(chunks: &mut VecDeque<(u8, bool)>, data: &mut VecDeque<u8>) -> (u8, bool) {
    if chunks.len() == 0 {
        let (len1, cont1, len2, cont2) = lengths_split(data.pop_front().unwrap());

        chunks.push_back((len1, cont1 != 0));
        chunks.push_back((len2, cont2 != 0));
    }

    chunks.pop_front().unwrap()
}
