//! Bencode data types.
//!
//! The types included in this module are:
//!
//! * [`Value`] - An enumeration over the different bencode data types.
//! * [`List`] - A list of bencode values.
//! * [`Dict`] - A **sorted** key-value object.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::str;
use std::str::Utf8Error;

use serde::de::Visitor;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::Deserialize;
use serde::Serialize;

/// A list of bencode values.
pub type List = Vec<Value>;

/// A **sorted** key-value map with keys that are UTF-8 valid strings.
pub type Dict = BTreeMap<String, Value>;

/// Represents any valid data type that can be encoded/decoded to and from bencode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A 64-bit signed integer.
    Int(i64),
    /// An array of bytes that may or **may not** be valid UTF-8.
    Text(Vec<u8>),
    /// A list of bencode values.
    List(List),
    /// A key-value map with keys that are UTF-8 valid strings.
    Dict(Dict),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Value::Int(int) => write!(f, "{}", int),
            Value::Text(ref bytes) => {
                let v = String::from_utf8_lossy(bytes);
                write!(f, "\"{}\"", &v)
            }
            Value::List(ref list) => {
                f.write_str("[")?;

                let mut first = true;
                for elem in list {
                    if !first {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", elem)?;
                    first = false;
                }

                f.write_str("]")
            }
            Value::Dict(ref dict) => {
                f.write_str("{")?;

                let mut first = true;
                for (key, val) in dict {
                    if first {
                        first = false;
                        f.write_str(" ")?;
                    } else {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}: {}", key, val)?;
                }

                if !first {
                    f.write_str(" ")?;
                }

                f.write_str("}")
            }
        }
    }
}

impl Value {
    /// Returns an `i64` if the value is an `Int`. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bende::Value;
    ///
    /// let val = Value::Int(50);
    /// assert_eq!(val.as_i64(), Some(50));
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Int(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a slice of bytes if the value is `Text`. Otherwise `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bende::Value;
    ///
    /// let val = Value::Text(b"foo".to_vec());
    /// assert_eq!(val.as_bytes(), Some(b"foo".as_slice()));
    /// ```
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match *self {
            Value::Text(ref v) => Some(v),
            _ => None,
        }
    }

    /// Returns a mutable reference to a slice of bytes if the value is `Text`. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bende::Value;
    ///
    /// let mut val = Value::Text(b"foo".to_vec());
    /// if let Some(bytes) = val.as_bytes_mut() {
    ///     bytes[0] = b'b';
    ///     assert_eq!(bytes, b"boo");
    /// }
    /// ```
    pub fn as_bytes_mut(&mut self) -> Option<&mut [u8]> {
        match *self {
            Value::Text(ref mut v) => Some(v),
            _ => None,
        }
    }

    /// Attempts to get a `str` from the value.
    ///
    /// Returns `None` if the value is not `Text`, or an error if the value is `Text` but the bytes are not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use bende::Value;
    ///
    /// let val = Value::Text(b"foo".to_vec());
    /// assert_eq!(val.as_str(), Ok(Some("foo")));
    /// ```
    pub fn as_str(&self) -> Result<Option<&str>, Utf8Error> {
        match *self {
            Value::Text(ref v) => str::from_utf8(v).map(Some),
            _ => Ok(None),
        }
    }

    /// Attempts to get a mutable reference to a `str` from the value.
    ///
    /// Returns `None` if the value is not `Text`, or an error if the value is `Text` but the bytes are not valid UTF-8.
    pub fn as_str_mut(&mut self) -> Result<Option<&mut str>, Utf8Error> {
        match *self {
            Value::Text(ref mut v) => str::from_utf8_mut(v).map(Some),
            _ => Ok(None),
        }
    }

    /// Returns a slice of values if the value is a `List`. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bende::Value;
    ///
    /// let val = Value::List(vec![Value::Int(50), Value::Text(b"foo".to_vec())]);
    /// for elem in val.as_list().unwrap() {
    ///     println!("{:?}", elem);
    /// }
    /// ```
    pub fn as_list(&self) -> Option<&[Value]> {
        match *self {
            Value::List(ref v) => Some(v),
            _ => None,
        }
    }

    /// Returns a mutable reference to a slice of values if the value is a `List`. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bende::Value;
    ///
    /// let mut val = Value::List(vec![Value::Int(50), Value::Int(50)]);
    /// for elem in val.as_list_mut().unwrap() {
    ///     *elem = Value::Text(b"foo".to_vec());
    /// }
    /// ```
    pub fn as_list_mut(&mut self) -> Option<&mut List> {
        match *self {
            Value::List(ref mut v) => Some(v),
            _ => None,
        }
    }

    /// Returns a `BTreeMap` if the value is a `Dict`. Otherwise, `None` is returned.
    pub fn as_dict(&self) -> Option<&Dict> {
        match *self {
            Value::Dict(ref v) => Some(v),
            _ => None,
        }
    }

    /// Returns a mutable reference to a `BTreeMap` if the value is a `Dict`. Otherwise, `None` is returned.
    pub fn as_dict_mut(&mut self) -> Option<&mut Dict> {
        match *self {
            Value::Dict(ref mut v) => Some(v),
            _ => None,
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Value::Int(v) => ser.serialize_i64(v),
            Value::Text(ref v) => ser.serialize_bytes(v),
            Value::List(ref v) => {
                let mut seq = ser.serialize_seq(Some(v.len()))?;
                for elem in v {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            Value::Dict(ref v) => {
                let mut map = ser.serialize_map(Some(v.len()))?;
                for (key, val) in v {
                    map.serialize_entry(key, val)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any valid bencode type")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
                Ok(Value::Int(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
                Ok(Value::Int(v as i64))
            }

            fn visit_str<E>(self, v: &str) -> Result<Value, E> {
                Ok(Value::Text(v.as_bytes().to_owned()))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Value, E> {
                Ok(Value::Text(v.into_bytes()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
                Ok(Value::Text(v.to_owned()))
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_bytes(v)
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                Ok(Value::Text(v))
            }

            fn visit_some<D>(self, de: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(de)
            }

            fn visit_seq<A>(self, mut access: A) -> Result<Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut list = Vec::new();
                while let Some(elem) = access.next_element()? {
                    list.push(elem);
                }
                Ok(Value::List(list))
            }

            fn visit_map<A>(self, mut access: A) -> Result<Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut dict = BTreeMap::new();
                while let Some((key, val)) = access.next_entry()? {
                    dict.insert(key, val);
                }
                Ok(Value::Dict(dict))
            }
        }

        de.deserialize_any(ValueVisitor)
    }
}

/// Implements `From<T> for Value` for any numerical type.
macro_rules! impl_value_from_num {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Value {
                fn from(v: $t) -> Value {
                    Value::Int(v as i64)
                }
            }
        )*
    }
}

// We need to skip i64.
impl_value_from_num!(u8, u16, u32, u64, usize, i8, i16, i32, isize);

// We do this manually as to avoid casting `i64 as i64`.
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Value::Text(v.to_owned())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.as_bytes().to_owned())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v.into_bytes())
    }
}

impl From<&[Value]> for Value {
    fn from(v: &[Value]) -> Self {
        Value::List(v.iter().cloned().collect())
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::List(v)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(v: HashMap<String, Value>) -> Self {
        Value::Dict(BTreeMap::from_iter(v.into_iter()))
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(v: BTreeMap<String, Value>) -> Self {
        Value::Dict(v)
    }
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};

    use super::Value;
    use crate::{decode, encode};

    #[test]
    fn encode_value_int() {
        let val = Value::Int(1995);
        assert_eq!(encode(&val).unwrap(), b"i1995e");
    }

    #[test]
    fn encode_value_text() {
        let val = Value::Text(b"foo".to_vec());
        assert_eq!(encode(&val).unwrap(), b"3:foo");
    }

    #[test]
    fn encode_value_list() {
        let val =
            Value::List(vec![Value::Int(1995), Value::Text(b"foo".to_vec())]);
        assert_eq!(encode(&val).unwrap(), b"li1995e3:fooe");
    }

    #[test]
    fn encode_value_dict() {
        let mut map = HashMap::new();
        map.insert("foo".to_string(), Value::Int(1995));
        map.insert("bar".to_string(), Value::Text(b"faz".to_vec()));

        assert_eq!(encode(&map).unwrap(), b"d3:bar3:faz3:fooi1995ee");
    }

    #[test]
    fn decode_value_int() {
        assert_eq!(decode::<Value>(b"i1995e").unwrap(), Value::Int(1995));
    }

    #[test]
    fn decode_value_text() {
        assert_eq!(
            decode::<Value>(b"3:foo").unwrap(),
            Value::Text(b"foo".to_vec())
        );
    }

    #[test]
    fn decode_value_list() {
        assert_eq!(
            decode::<Value>(b"li1995e3:fooe").unwrap(),
            Value::List(vec![Value::Int(1995), Value::Text(b"foo".to_vec())])
        )
    }

    #[test]
    fn decode_value_dict() {
        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), Value::Int(1995));
        map.insert("bar".to_string(), Value::Text(b"faz".to_vec()));
        assert_eq!(
            decode::<Value>(b"d3:bar3:faz3:fooi1995ee").unwrap(),
            Value::Dict(map)
        )
    }
}
