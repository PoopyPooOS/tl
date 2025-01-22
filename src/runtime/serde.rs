use super::types::Value;
use serde::{
    de::{self, Expected, IntoDeserializer, MapAccess, SeqAccess, Visitor},
    forward_to_deserialize_any, Deserializer,
};
use std::collections::btree_map;

impl<'de> Deserializer<'de> for Value {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Boolean(b) => visitor.visit_bool(b),
            Value::Int(n) => visitor.visit_i64(
                n.try_into()
                    .map_err(|_| de::Error::custom("Integer overflowed"))?,
            ),
            Value::Float(f) => visitor.visit_f64(f),
            Value::String(s) => visitor.visit_string(s),
            Value::Array(arr) => {
                let seq = ValueSeq {
                    iter: arr.into_iter(),
                };
                visitor.visit_seq(seq)
            }
            Value::Object(map) => {
                let map = ValueMap {
                    iter: map.into_iter(),
                    value: None,
                };
                visitor.visit_map(map)
            }
            Value::Function { .. } => Err(de::Error::custom("Functions cannot be deserialized")),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(s) => visitor.visit_enum(s.into_deserializer()),
            _ => Err(de::Error::invalid_type(de::Unexpected::Unit, &self)),
        }
    }

    // Forward other methods to deserialize_any
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl Expected for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(self.type_of())
    }
}

struct ValueSeq {
    iter: std::vec::IntoIter<Value>,
}

impl<'de> SeqAccess<'de> for ValueSeq {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

struct ValueMap {
    iter: btree_map::IntoIter<String, Value>,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for ValueMap {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(de::Error::custom("Value expected after key")),
        }
    }
}
