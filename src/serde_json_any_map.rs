extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use serde::ser::{Serialize, Serializer, SerializeMap, Error};

pub struct SerializeAnyKeyMapWrapper<'a,K,V>
{
  map: &'a HashMap<K,V>
}

impl<'a, K,V> Serialize for SerializeAnyKeyMapWrapper<'a, K,V> where
  K: Serialize,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let base = self.map;
    let mut ser_map = serializer.serialize_map(Some(base.len()))?;
    for (k, v) in base {
      ser_map.serialize_entry(match &serde_json::to_string(k)
      {
        Ok(key_string) => key_string,
        Err(e) => {return Err(e).map_err(S::Error::custom); }
      }, v)?;
    }
    ser_map.end()
  }
}

pub fn to_string<K,V>(map: &HashMap<K,V>) -> Result<String, serde_json::Error> where
K: Serialize,
V: Serialize
{
  let wrap = SerializeAnyKeyMapWrapper {
    map: map
  };
  serde_json::to_string(&wrap)
}