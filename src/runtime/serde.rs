use crate::runtime::ValueKind;

use super::types::Value;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Expected, IntoDeserializer, MapAccess, SeqAccess, Visitor},
    forward_to_deserialize_any,
};
use std::{collections::btree_map, fmt};

impl<'de> Deserializer<'de> for Value {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.kind {
            ValueKind::Null => visitor.visit_unit(),
            ValueKind::Boolean(val) => visitor.visit_bool(val),
            ValueKind::Int(val) => visitor.visit_i64(
                val.try_into()
                    .map_err(|_| de::Error::custom("Integer overflowed"))?,
            ),
            ValueKind::Float(val) => visitor.visit_f64(val),
            ValueKind::String(val) => visitor.visit_string(val),
            ValueKind::Path(val) => visitor.visit_string(val.display().to_string()),
            ValueKind::Array(arr) => {
                let seq = ValueSeq {
                    iter: arr.into_iter(),
                };
                visitor.visit_seq(seq)
            }
            ValueKind::Object(map) => {
                let map = ValueMap {
                    iter: map.into_iter(),
                    value: None,
                };
                visitor.visit_map(map)
            }
            ValueKind::Function { .. } => {
                Err(de::Error::custom("Functions cannot be deserialized"))
            }
            ValueKind::Builtin(..) => Err(de::Error::custom("Builtins cannot be deserialized")),
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
        match self.kind {
            ValueKind::String(s) => visitor.visit_enum(s.into_deserializer()),
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
                seed.deserialize(Value::new_builtin(ValueKind::String(key)))
                    .map(Some)
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

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.kind {
            ValueKind::Boolean(v) => serializer.serialize_bool(*v),
            ValueKind::Int(v) => serializer.serialize_i64(*v as i64),
            ValueKind::Float(v) => serializer.serialize_f64(*v),
            ValueKind::String(v) => serializer.serialize_str(v),
            ValueKind::Path(v) => serializer.serialize_str(&v.display().to_string()),
            ValueKind::Array(v) => v.serialize(serializer),
            ValueKind::Object(v) => v.serialize(serializer),
            ValueKind::Null | ValueKind::Function { .. } | ValueKind::Builtin(..) => {
                serializer.serialize_unit()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid Value")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Null))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Null))
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Boolean(v)))
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Int(v as isize)))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Float(f64::from(v))))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::Float(v)))
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::String(v.to_string())))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::String(v.to_owned())))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
                Ok(Value::new_builtin(ValueKind::String(v)))
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                seq: A,
            ) -> Result<Self::Value, A::Error> {
                let vec = Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(Value::new_builtin(ValueKind::Array(vec)))
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                let map = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(Value::new_builtin(ValueKind::Object(map)))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
