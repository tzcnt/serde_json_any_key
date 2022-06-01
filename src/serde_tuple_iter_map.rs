extern crate serde;
extern crate serde_json;

use serde::ser::{Serialize, Serializer, SerializeMap, Error};
use std::cell::RefCell;

pub struct SerializeAnyIterWrapper<'a, K, V>
{
  pub iter: RefCell<&'a mut (dyn Iterator<Item=(K, V)> + 'a)>
}

impl<'a, K, V> Serialize for SerializeAnyIterWrapper<'a, K, V> where
  K: Serialize,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    while let Some((k, v)) = iter.next() {
      ser_map.serialize_entry(match &serde_json::to_string(&k)
      {
        Ok(key_string) => key_string,
        Err(e) => { return Err(e).map_err(S::Error::custom); }
      }, &v)?;
    }
    ser_map.end()
  }
}

pub fn to_string<'a, K,V>(iter: &'a mut dyn Iterator<Item=(K,V)>) -> Result<String, serde_json::Error> where
K: Serialize,
V: Serialize
{
  serde_json::to_string(&SerializeAnyIterWrapper {
    iter: RefCell::new(iter)
  })
}
