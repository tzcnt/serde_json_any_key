extern crate serde;
extern crate serde_json;

use core::marker::PhantomData;
use std::hash::Hash;
use serde_json::{Map, Value};
use serde::ser::{Serialize, Serializer, SerializeMap, Error};
use serde::de::{Deserialize, DeserializeOwned};
use std::cell::RefCell;
struct SerializeMapIterWrapper<'a, K, V>
{
  pub iter: RefCell<&'a mut (dyn Iterator<Item=(&'a K, &'a V)> + 'a)>
}

impl<'a, K, V> Serialize for SerializeMapIterWrapper<'a, K, V> where
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

/// Serialize an Iterator<(&K, &V)> like that given by HashMap::iter().
/// serde_json::to_string() will be called on each K element during serialization.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use serde::Serialize;
/// use serde_json::Error;
/// 
/// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let mut map = HashMap::<Test, Test>::new();
/// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
/// 
/// // Naive serde_json cannot serialize this map.
/// let ser1 = serde_json::to_string(&map);
/// assert_eq!(ser1.err().unwrap().to_string(), "key must be a string");
/// 
/// // Use this crate's utility function - elements are serialized lazily.
/// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
/// let ser2 = serde_json_tuple_iter::map_to_json(&mut map.iter()).unwrap();
///
/// // Compare to a winded workaround that copies the map.
/// // Same output
/// let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
/// let ser3 = serde_json::to_string(&string_map).unwrap();
///
/// assert_eq!(ser2, ser3);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn map_to_json<'a, K, V>(iter: &'a mut dyn Iterator<Item=(&'a K, &'a V)>) -> Result<String, serde_json::Error> where
K: Serialize,
V: Serialize
{
  serde_json::to_string(&SerializeMapIterWrapper {
    iter: RefCell::new(iter)
  })
}

pub struct DeserializeMapIter<'a, K, V> where
K: Deserialize<'a>,
V: Deserialize<'a>
{
  str: &'a str,
  iter: serde_json::map::Iter<'a>,
  keyType: PhantomData<K>,
  valType: PhantomData<V>
}

// impl<'a, K, V> DeserializeMapIter<'a, K, V> {
//   pub fn new(json: serde_json::Value) -> DeserializeMapIter<'a, K, V> {
//     DeserializeMapIter<'a, K, V> {
//       json: 
//     }
//   }
// }

impl<'a, K, V> Iterator for DeserializeMapIter<'a, K, V> where
K: Deserialize<'a>,
V: Deserialize<'a>
{
  type Item = (K, V);
  fn next(&mut self) -> Option<Self::Item> {
    if let Some(next) = self.iter.next() {
      let keyObj: K = serde_json::from_str(next.0).unwrap();
      let valObj: V = <V as Deserialize>::deserialize(next.1).unwrap();
      Some((keyObj, valObj))
    } else {
      return None;
    }
  }
}
fn jsonItemKVMapFunc<K, V>(kv: (&String, &serde_json::Value)) -> Option<(K,V)> where
K: DeserializeOwned,
V: DeserializeOwned
{
  let keyObj: K = serde_json::from_str(kv.0).unwrap();
  let valObj: V = <V as Deserialize>::deserialize(kv.1).unwrap();
  Some((keyObj, valObj))
}

//pub fn json_to_map<'a, K, V>(str: &'a str) -> Result<DeserializeMapIter<'a, K, V>, serde_json::Error> where
//std::iter::Map<serde_json::map::Iter<'_>, fn((&std::string::String, &Value)) -> std::option::Option<(K, V)>>
pub fn json_to_map<'a, K, V>(str: &'a str) -> std::collections::HashMap<K, V> where
//pub fn json_to_map<'a, K, V>(str: &'a str) -> std::iter::Map<serde_json::map::Iter<'_>, fn((&std::string::String, &Value)) -> Option<(K, V)>> where
for<'de> K: Deserialize<'de> + std::cmp::Eq + Hash,
for<'de> V: Deserialize<'de>
{
  //serde_json::Value::from(str).as_object().unwrap().into_iter().map(jsonItemKVMapFunc)

  // let map: std::collections::HashMap<K, V> = serde_json::Value::from(str).as_object().unwrap().into_iter().map(|(k, v)| {
  //   let keyObj: K = serde_json::from_str(k).unwrap();
  //   let valObj: V = <V as Deserialize>::deserialize(v).unwrap();
  //   Some((keyObj, valObj))
  // }).collect();
  //Ok(x)
  //let contents_json: serde_json::Value = serde_json::Value::from(str)?;
  let mut map: std::collections::HashMap<K, V> = std::collections::HashMap::new();
  let v = serde_json::Value::from(str);
  let o = v.as_object().unwrap();
  for (key, val) in o.iter() {
    let key_obj: K = serde_json::from_str(key).unwrap();
    let val_obj: V = <V as Deserialize>::deserialize(val).unwrap();
    map.insert(key_obj, val_obj);
  }
  map
}

struct SerializeVecIterWrapper<'a, K, V>
{
  pub iter: RefCell<&'a mut (dyn Iterator<Item=&'a (K, V)> + 'a)>
}

impl<'a, K, V> Serialize for SerializeVecIterWrapper<'a, K, V> where
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

/// Serialize an Iterator<&(K, V)> like that given by Vec::<(K, V)>::iter().
/// serde_json::to_string() will be called on each K element during serialization.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use serde::Serialize;
/// use serde_json::Error;
/// 
/// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let v = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
/// 
/// // Naive serde_json will serialize this as an array, not a map.
/// // Outputs [[{"a":3,"b":5},{"a":7,"b":9}]]
/// let ser1 = serde_json::to_string(&v).unwrap();
/// assert_eq!(ser1, "[[{\"a\":3,\"b\":5},{\"a\":7,\"b\":9}]]");
/// 
/// // Use this crate's utility function - elements are serialized lazily.
/// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
/// let ser2 = serde_json_tuple_iter::vec_to_json(&mut v.iter()).unwrap();
///
/// assert_eq!(ser2, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn vec_to_json<'a, K, V>(iter: &'a mut dyn Iterator<Item=&'a (K, V)>) -> Result<String, serde_json::Error> where
K: Serialize,
V: Serialize
{
  serde_json::to_string(&SerializeVecIterWrapper {
    iter: RefCell::new(iter)
  })
}